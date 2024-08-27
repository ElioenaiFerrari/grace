package pipeline

import (
	"context"

	"github.com/google/uuid"
	"github.com/rotisserie/eris"
	"github.com/sashabaranov/go-openai"
)

type Agent struct {
	system openai.ChatCompletionMessage
	apiKey string
	id     string
	name   string
	client *openai.Client
}

func NewAgent(apiKey, name string) AgentBehavior {
	return &Agent{
		apiKey: apiKey,
		name:   name,
		id:     uuid.NewString(),
		client: openai.NewClient(apiKey),
	}
}

func (a *Agent) SetSystemMessage(ctx context.Context, message string) {
	a.system = openai.ChatCompletionMessage{
		Role:    openai.ChatMessageRoleSystem,
		Content: message,
	}
}

func (a *Agent) ID() string {
	return a.id
}

func (a *Agent) Name() string {
	return a.name
}

func (a *Agent) ChatCompletion(ctx context.Context, messages []openai.ChatCompletionMessage) (*openai.ChatCompletionMessage, error) {
	messages = append([]openai.ChatCompletionMessage{a.system}, messages...)

	req := openai.ChatCompletionRequest{
		Model:     openai.GPT4oMini,
		MaxTokens: 30,
		Messages:  messages,
	}

	res, err := a.client.CreateChatCompletion(ctx, req)
	if err != nil {
		return nil, eris.Wrap(err, "failed to create chat completion")
	}

	return &res.Choices[0].Message, nil
}
