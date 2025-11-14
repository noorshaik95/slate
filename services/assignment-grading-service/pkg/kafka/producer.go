package kafka

import (
	"context"
	"encoding/json"
	"fmt"
	"time"

	"github.com/segmentio/kafka-go"
)

// EventPublisher defines the interface for publishing events
type EventPublisher interface {
	PublishEvent(ctx context.Context, event Event) error
	Close() error
}

// Producer wraps the Kafka writer
type Producer struct {
	writer  *kafka.Writer
	enabled bool
}

// NewProducer creates a new Kafka producer
func NewProducer(brokers []string, topic string, enabled bool) *Producer {
	if !enabled {
		return &Producer{enabled: false}
	}

	writer := &kafka.Writer{
		Addr:         kafka.TCP(brokers...),
		Topic:        topic,
		Balancer:     &kafka.LeastBytes{},
		MaxAttempts:  3,
		BatchSize:    1,
		BatchTimeout: 10 * time.Millisecond,
		ReadTimeout:  10 * time.Second,
		WriteTimeout: 10 * time.Second,
		RequiredAcks: kafka.RequireOne,
		Compression:  kafka.Snappy,
	}

	return &Producer{
		writer:  writer,
		enabled: true,
	}
}

// PublishEvent publishes an event to Kafka
func (p *Producer) PublishEvent(ctx context.Context, event Event) error {
	if !p.enabled {
		// Kafka is disabled, silently skip
		return nil
	}

	// Marshal event to JSON
	value, err := json.Marshal(event)
	if err != nil {
		return fmt.Errorf("failed to marshal event: %w", err)
	}

	// Create Kafka message
	message := kafka.Message{
		Key:   []byte(event.AggregateID),
		Value: value,
		Time:  event.Timestamp,
	}

	// Write message with retries
	if err := p.writer.WriteMessages(ctx, message); err != nil {
		return fmt.Errorf("failed to write message to Kafka: %w", err)
	}

	return nil
}

// Close closes the Kafka writer
func (p *Producer) Close() error {
	if p.writer != nil {
		return p.writer.Close()
	}
	return nil
}
