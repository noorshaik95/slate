package storage

import (
	"bytes"
	"io"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestNewLocalFileStorage(t *testing.T) {
	tempDir := t.TempDir()

	t.Run("creates directory if not exists", func(t *testing.T) {
		newDir := filepath.Join(tempDir, "new_storage")
		storage, err := NewLocalFileStorage(newDir, 1024*1024)

		if err != nil {
			t.Fatalf("unexpected error: %v", err)
		}

		if storage == nil {
			t.Fatal("expected storage to be created")
		}

		// Check directory exists
		if _, err := os.Stat(newDir); os.IsNotExist(err) {
			t.Error("expected directory to be created")
		}
	})

	t.Run("uses existing directory", func(t *testing.T) {
		storage, err := NewLocalFileStorage(tempDir, 1024*1024)

		if err != nil {
			t.Fatalf("unexpected error: %v", err)
		}

		if storage == nil {
			t.Fatal("expected storage to be created")
		}
	})
}

func TestLocalFileStorage_Save(t *testing.T) {
	tempDir := t.TempDir()
	storage, err := NewLocalFileStorage(tempDir, 1024) // 1KB limit
	if err != nil {
		t.Fatalf("failed to create storage: %v", err)
	}

	t.Run("save valid PDF file", func(t *testing.T) {
		content := []byte("fake PDF content")
		reader := bytes.NewReader(content)

		path, err := storage.Save("test.pdf", reader, "application/pdf")

		if err != nil {
			t.Fatalf("unexpected error: %v", err)
		}

		if path == "" {
			t.Fatal("expected non-empty path")
		}

		// Verify file exists
		fullPath := filepath.Join(tempDir, path)
		if _, err := os.Stat(fullPath); os.IsNotExist(err) {
			t.Error("expected file to be saved")
		}

		// Verify content
		savedContent, err := os.ReadFile(fullPath)
		if err != nil {
			t.Fatalf("failed to read saved file: %v", err)
		}

		if !bytes.Equal(savedContent, content) {
			t.Error("saved content does not match original")
		}
	})

	t.Run("save valid DOCX file", func(t *testing.T) {
		content := []byte("fake DOCX content")
		reader := bytes.NewReader(content)

		path, err := storage.Save("document.docx", reader, "application/vnd.openxmlformats-officedocument.wordprocessingml.document")

		if err != nil {
			t.Fatalf("unexpected error: %v", err)
		}

		if !strings.HasSuffix(path, ".docx") {
			t.Errorf("expected path to end with .docx, got %s", path)
		}
	})

	t.Run("reject disallowed content type", func(t *testing.T) {
		content := []byte("fake executable")
		reader := bytes.NewReader(content)

		_, err := storage.Save("malicious.exe", reader, "application/x-msdownload")

		if err == nil {
			t.Fatal("expected error for disallowed content type")
		}

		if !strings.Contains(err.Error(), "not allowed") {
			t.Errorf("unexpected error message: %v", err)
		}
	})

	t.Run("reject file exceeding size limit", func(t *testing.T) {
		content := make([]byte, 2048) // 2KB, exceeds 1KB limit
		reader := bytes.NewReader(content)

		_, err := storage.Save("large.pdf", reader, "application/pdf")

		if err == nil {
			t.Fatal("expected error for file exceeding size limit")
		}

		if !strings.Contains(err.Error(), "exceeds maximum allowed size") {
			t.Errorf("unexpected error message: %v", err)
		}
	})

	t.Run("sanitize dangerous filename", func(t *testing.T) {
		content := []byte("test content")
		reader := bytes.NewReader(content)

		path, err := storage.Save("../../../etc/passwd.pdf", reader, "application/pdf")

		if err != nil {
			t.Fatalf("unexpected error: %v", err)
		}

		// Path should not contain ".."
		if strings.Contains(path, "..") {
			t.Error("path should not contain '..'")
		}

		// Path should not contain "/"
		if strings.Contains(path, "/") {
			t.Error("path should not contain '/'")
		}
	})

	t.Run("handle empty filename", func(t *testing.T) {
		content := []byte("test content")
		reader := bytes.NewReader(content)

		_, err := storage.Save("", reader, "application/pdf")

		if err == nil {
			t.Fatal("expected error for empty filename")
		}
	})

	t.Run("preserve file extension", func(t *testing.T) {
		content := []byte("test content")
		reader := bytes.NewReader(content)

		path, err := storage.Save("assignment.pdf", reader, "application/pdf")

		if err != nil {
			t.Fatalf("unexpected error: %v", err)
		}

		if !strings.HasSuffix(path, ".pdf") {
			t.Errorf("expected path to end with .pdf, got %s", path)
		}
	})
}

func TestLocalFileStorage_Get(t *testing.T) {
	tempDir := t.TempDir()
	storage, err := NewLocalFileStorage(tempDir, 1024*1024)
	if err != nil {
		t.Fatalf("failed to create storage: %v", err)
	}

	t.Run("get existing file", func(t *testing.T) {
		// Save a file first
		content := []byte("test content for retrieval")
		savePath, err := storage.Save("test.pdf", bytes.NewReader(content), "application/pdf")
		if err != nil {
			t.Fatalf("failed to save file: %v", err)
		}

		// Retrieve it
		reader, err := storage.Get(savePath)
		if err != nil {
			t.Fatalf("unexpected error: %v", err)
		}
		defer reader.Close()

		// Read and compare
		retrieved, err := io.ReadAll(reader)
		if err != nil {
			t.Fatalf("failed to read retrieved file: %v", err)
		}

		if !bytes.Equal(retrieved, content) {
			t.Error("retrieved content does not match original")
		}
	})

	t.Run("get non-existent file", func(t *testing.T) {
		_, err := storage.Get("non-existent-file.pdf")

		if err == nil {
			t.Fatal("expected error for non-existent file")
		}

		if !strings.Contains(err.Error(), "not found") {
			t.Errorf("unexpected error message: %v", err)
		}
	})

	t.Run("prevent path traversal on get", func(t *testing.T) {
		_, err := storage.Get("../../../etc/passwd")

		if err == nil {
			t.Fatal("expected error for path traversal attempt")
		}

		if !strings.Contains(err.Error(), "invalid file path") {
			t.Errorf("unexpected error message: %v", err)
		}
	})
}

func TestLocalFileStorage_Delete(t *testing.T) {
	tempDir := t.TempDir()
	storage, err := NewLocalFileStorage(tempDir, 1024*1024)
	if err != nil {
		t.Fatalf("failed to create storage: %v", err)
	}

	t.Run("delete existing file", func(t *testing.T) {
		// Save a file first
		content := []byte("test content")
		savePath, err := storage.Save("test.pdf", bytes.NewReader(content), "application/pdf")
		if err != nil {
			t.Fatalf("failed to save file: %v", err)
		}

		// Delete it
		err = storage.Delete(savePath)
		if err != nil {
			t.Fatalf("unexpected error: %v", err)
		}

		// Verify it's deleted
		fullPath := filepath.Join(tempDir, savePath)
		if _, err := os.Stat(fullPath); !os.IsNotExist(err) {
			t.Error("expected file to be deleted")
		}
	})

	t.Run("delete non-existent file (no error)", func(t *testing.T) {
		err := storage.Delete("non-existent-file.pdf")

		// Should not error when file doesn't exist
		if err != nil {
			t.Errorf("unexpected error for non-existent file: %v", err)
		}
	})

	t.Run("prevent path traversal on delete", func(t *testing.T) {
		err := storage.Delete("../../../etc/passwd")

		if err == nil {
			t.Fatal("expected error for path traversal attempt")
		}

		if !strings.Contains(err.Error(), "invalid file path") {
			t.Errorf("unexpected error message: %v", err)
		}
	})
}

func TestLocalFileStorage_Exists(t *testing.T) {
	tempDir := t.TempDir()
	storage, err := NewLocalFileStorage(tempDir, 1024*1024)
	if err != nil {
		t.Fatalf("failed to create storage: %v", err)
	}

	t.Run("existing file returns true", func(t *testing.T) {
		content := []byte("test content")
		savePath, err := storage.Save("test.pdf", bytes.NewReader(content), "application/pdf")
		if err != nil {
			t.Fatalf("failed to save file: %v", err)
		}

		exists, err := storage.Exists(savePath)
		if err != nil {
			t.Fatalf("unexpected error: %v", err)
		}

		if !exists {
			t.Error("expected file to exist")
		}
	})

	t.Run("non-existent file returns false", func(t *testing.T) {
		exists, err := storage.Exists("non-existent-file.pdf")
		if err != nil {
			t.Fatalf("unexpected error: %v", err)
		}

		if exists {
			t.Error("expected file to not exist")
		}
	})

	t.Run("prevent path traversal on exists", func(t *testing.T) {
		_, err := storage.Exists("../../../etc/passwd")

		if err == nil {
			t.Fatal("expected error for path traversal attempt")
		}
	})
}

func TestSanitizeFilename(t *testing.T) {
	tests := []struct {
		name     string
		input    string
		expected string
	}{
		{
			name:     "normal filename",
			input:    "document.pdf",
			expected: "document.pdf",
		},
		{
			name:     "path traversal attempt",
			input:    "../../../etc/passwd",
			expected: "......etcpasswd",
		},
		{
			name:     "null bytes",
			input:    "file\x00name.pdf",
			expected: "filename.pdf",
		},
		{
			name:     "special characters",
			input:    "my document!@#$%^&*().pdf",
			expected: "my_document__________.pdf",
		},
		{
			name:     "windows path",
			input:    "C:\\Windows\\System32\\file.pdf",
			expected: "CWindowsSystem32file.pdf",
		},
		{
			name:     "very long filename",
			input:    strings.Repeat("a", 300) + ".pdf",
			expected: strings.Repeat("a", 251) + ".pdf", // Truncated to 255 chars
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := sanitizeFilename(tt.input)
			if result != tt.expected {
				t.Errorf("expected %q but got %q", tt.expected, result)
			}
		})
	}
}

func TestIsAllowedContentType(t *testing.T) {
	tests := []struct {
		contentType string
		expected    bool
	}{
		{"application/pdf", true},
		{"application/vnd.openxmlformats-officedocument.wordprocessingml.document", true},
		{"application/msword", true},
		{"text/plain", true},
		{"application/zip", true},
		{"application/x-msdownload", false},
		{"application/javascript", false},
		{"text/html", false},
		{"image/jpeg", false},
		{"", false},
	}

	for _, tt := range tests {
		t.Run(tt.contentType, func(t *testing.T) {
			result := isAllowedContentType(tt.contentType)
			if result != tt.expected {
				t.Errorf("isAllowedContentType(%q) = %v, want %v", tt.contentType, result, tt.expected)
			}
		})
	}
}
