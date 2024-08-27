package pipeline

import (
	"context"

	"github.com/sashabaranov/go-openai"
)

type AgentBehavior interface {
	ID() string
	Name() string
	ChatCompletion(ctx context.Context, messages []openai.ChatCompletionMessage) (*openai.ChatCompletionMessage, error)
	SetSystemMessage(ctx context.Context, message string)
}
