package pipeline

import (
	"context"
	"fmt"
	"log"
	"time"

	"github.com/sashabaranov/go-openai"
)

type UndirectedGraph struct {
	connections map[string][]string
}

func NewUndirectedGraph() *UndirectedGraph {
	return &UndirectedGraph{
		connections: map[string][]string{},
	}
}

func (g *UndirectedGraph) AddConnection(from, to string) {
	g.connections[from] = append(g.connections[from], to)
	g.connections[to] = append(g.connections[to], from)
}

func (g *UndirectedGraph) ConversationBFS(ctx context.Context, agents map[string]AgentBehavior) (chan *openai.ChatCompletionMessage, error) {
	ch := make(chan *openai.ChatCompletionMessage)

	go func() {
		messages := []openai.ChatCompletionMessage{
			{
				Role:    openai.ChatMessageRoleUser,
				Content: "Olá tudo bem? Se apresentem e digam o que fazem e o precisam. As mensagens devem ser seguidas de <Profissão>: <Mensagem>",
			},
		}
		for {
			fmt.Printf("Running conversation, %d\n", len(messages))
			visited := map[string]bool{}
			queue := []string{}

			for id := range agents {
				queue = append(queue, id)
				visited[id] = true
			}

			for len(queue) > 0 {
				id := queue[0]
				queue = queue[1:]

				agent := agents[id]
				message, err := agent.ChatCompletion(ctx, messages)
				if err != nil {
					log.Printf("failed to chat completion: %s", err)
					continue
				}

				messages = append(messages, *message)
				ch <- message

				for _, to := range g.connections[id] {
					if visited[to] {
						continue
					}

					queue = append(queue, to)
					visited[to] = true
				}
			}

			time.Sleep(1 * time.Second)
		}
	}()

	return ch, nil
}

type Room struct {
	agents      []AgentBehavior
	connections *UndirectedGraph
}

func NewRoom() *Room {
	return &Room{
		agents:      []AgentBehavior{},
		connections: NewUndirectedGraph(),
	}
}

func (r *Room) AddConnection(from, to AgentBehavior) {
	r.agents = append(r.agents, from)
	r.connections.AddConnection(from.ID(), to.ID())
}

func (r *Room) findAgentByID(id string) AgentBehavior {
	for _, agent := range r.agents {
		if agent.ID() == id {
			return agent
		}
	}

	return nil
}

func (r *Room) Run(ctx context.Context) (chan *openai.ChatCompletionMessage, error) {
	agents := map[string]AgentBehavior{}
	for _, agent := range r.agents {
		agents[agent.ID()] = agent
	}

	return r.connections.ConversationBFS(ctx, agents)
}
