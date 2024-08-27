package pipeline

import (
	"context"
	"log"

	"github.com/sashabaranov/go-openai"
)

type Room struct {
	connections *Graph
}

func NewRoom() *Room {
	return &Room{
		connections: NewGraph(Undirected),
	}
}

func (r *Room) AddConnection(from, to AgentBehavior) {
	r.connections.AddEdge(Edge{
		From:      from.ID(),
		To:        to.ID(),
		FromAgent: from,
		ToAgent:   to,
	})

	log.Println("added connection", from.Name(), "to", to.Name())
}

func (r *Room) Print() {
	r.connections.Print()
}

func (r *Room) Run(ctx context.Context) (chan *openai.ChatCompletionMessage, error) {
	ch := make(chan *openai.ChatCompletionMessage)

	go func() {
		log.Println("running room")
		defer close(ch)

		messages := []openai.ChatCompletionMessage{
			{
				Role:    openai.ChatMessageRoleSystem,
				Content: "Olá, se apresentem e conversem sobre o projeto. O projeto será um jogo de cobrinha em Rust.",
			},
		}

		for {
			for _, node := range r.connections.Nodes {
				agent := node.Agent

				message, err := agent.ChatCompletion(ctx, messages)
				if err != nil {
					log.Println("failed to chat completion", err)
					return
				}

				ch <- message
				messages = append(messages, *message)
			}
		}
	}()

	return ch, nil
}
