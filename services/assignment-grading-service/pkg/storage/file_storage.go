package storage

import (
	"fmt"
	"io"
	"os"
	"path/filepath"
	"regexp"
	"strings"

	"github.com/google/uuid"
)

// FileStorage defines the interface for file storage operations
type FileStorage interface {
	Save(filename string, content io.Reader, contentType string) (string, error)
	Get(filePath string) (io.ReadCloser, error)
	Delete(filePath string) error
	Exists(filePath string) (bool, error)
}

// LocalFileStorage implements FileStorage for local filesystem
type LocalFileStorage struct {
	basePath    string
	maxFileSize int64 // in bytes
}

// NewLocalFileStorage creates a new local file storage instance
func NewLocalFileStorage(basePath string, maxFileSize int64) (*LocalFileStorage, error) {
	// Create base directory if it doesn't exist
	if err := os.MkdirAll(basePath, 0o755); err != nil {
		return nil, fmt.Errorf("failed to create storage directory: %w", err)
	}

	return &LocalFileStorage{
		basePath:    basePath,
		maxFileSize: maxFileSize,
	}, nil
}

// Save stores a file and returns its path
func (s *LocalFileStorage) Save(filename string, content io.Reader, contentType string) (string, error) {
	// Sanitize filename
	sanitizedName := sanitizeFilename(filename)
	if sanitizedName == "" {
		return "", fmt.Errorf("invalid filename")
	}

	// Validate content type
	if !isAllowedContentType(contentType) {
		return "", fmt.Errorf("content type %s is not allowed", contentType)
	}

	// Generate unique filename to prevent collisions
	ext := filepath.Ext(sanitizedName)
	uniqueID := uuid.New().String()
	uniqueFilename := fmt.Sprintf("%s%s", uniqueID, ext)

	// Create full path
	fullPath := filepath.Join(s.basePath, uniqueFilename)

	// Ensure the path is within basePath (prevent directory traversal)
	if !strings.HasPrefix(fullPath, s.basePath) {
		return "", fmt.Errorf("invalid file path")
	}

	// Create the file
	file, err := os.Create(fullPath)
	if err != nil {
		return "", fmt.Errorf("failed to create file: %w", err)
	}
	defer file.Close()

	// Copy content with size limit
	limitedReader := io.LimitReader(content, s.maxFileSize+1)
	written, err := io.Copy(file, limitedReader)
	if err != nil {
		os.Remove(fullPath) // Clean up on error
		return "", fmt.Errorf("failed to write file: %w", err)
	}

	// Check if file size exceeds limit
	if written > s.maxFileSize {
		os.Remove(fullPath) // Clean up
		return "", fmt.Errorf("file size exceeds maximum allowed size of %d bytes", s.maxFileSize)
	}

	return uniqueFilename, nil
}

// Get retrieves a file
func (s *LocalFileStorage) Get(filePath string) (io.ReadCloser, error) {
	fullPath := filepath.Join(s.basePath, filePath)

	// Ensure the path is within basePath (prevent directory traversal)
	if !strings.HasPrefix(fullPath, s.basePath) {
		return nil, fmt.Errorf("invalid file path")
	}

	file, err := os.Open(fullPath)
	if err != nil {
		if os.IsNotExist(err) {
			return nil, fmt.Errorf("file not found")
		}
		return nil, fmt.Errorf("failed to open file: %w", err)
	}

	return file, nil
}

// Delete removes a file
func (s *LocalFileStorage) Delete(filePath string) error {
	fullPath := filepath.Join(s.basePath, filePath)

	// Ensure the path is within basePath (prevent directory traversal)
	if !strings.HasPrefix(fullPath, s.basePath) {
		return fmt.Errorf("invalid file path")
	}

	if err := os.Remove(fullPath); err != nil {
		if os.IsNotExist(err) {
			return nil // Already deleted, not an error
		}
		return fmt.Errorf("failed to delete file: %w", err)
	}

	return nil
}

// Exists checks if a file exists
func (s *LocalFileStorage) Exists(filePath string) (bool, error) {
	fullPath := filepath.Join(s.basePath, filePath)

	// Ensure the path is within basePath (prevent directory traversal)
	if !strings.HasPrefix(fullPath, s.basePath) {
		return false, fmt.Errorf("invalid file path")
	}

	_, err := os.Stat(fullPath)
	if err != nil {
		if os.IsNotExist(err) {
			return false, nil
		}
		return false, err
	}

	return true, nil
}

// sanitizeFilename removes unsafe characters from filename
func sanitizeFilename(filename string) string {
	// Remove path separators, colons, and null bytes
	filename = strings.ReplaceAll(filename, "/", "")
	filename = strings.ReplaceAll(filename, "\\", "")
	filename = strings.ReplaceAll(filename, ":", "")
	filename = strings.ReplaceAll(filename, "\x00", "")

	// Only allow alphanumeric, dash, underscore, and dot
	reg := regexp.MustCompile(`[^a-zA-Z0-9._-]`)
	filename = reg.ReplaceAllString(filename, "_")

	// Limit filename length
	if len(filename) > 255 {
		ext := filepath.Ext(filename)
		filename = filename[:255-len(ext)] + ext
	}

	return filename
}

// isAllowedContentType checks if the content type is allowed
func isAllowedContentType(contentType string) bool {
	allowedTypes := []string{
		"application/pdf",
		"application/vnd.openxmlformats-officedocument.wordprocessingml.document", // .docx
		"application/msword", // .doc
		"text/plain",
		"application/zip",
	}

	for _, allowed := range allowedTypes {
		if contentType == allowed {
			return true
		}
	}

	return false
}
