package pipeline

import (
	"log"
)

type GraphType = string

const (
	Undirected GraphType = "undirected"
	Directed   GraphType = "directed"
)

type Node struct {
	ID    string
	Agent AgentBehavior
}

type Edge struct {
	From      string
	FromAgent AgentBehavior
	To        string
	ToAgent   AgentBehavior
}

type Graph struct {
	Type  GraphType
	Nodes []Node
	Edges []Edge
}

func NewGraph(t GraphType) *Graph {
	return &Graph{
		Type:  t,
		Nodes: []Node{},
		Edges: []Edge{},
	}
}

func (g *Graph) Print() {
	log.Println("printing graph")
	for _, node := range g.Nodes {
		log.Println("node", node.ID)
	}

	for _, edge := range g.Edges {
		log.Println("edge", edge.From, "to", edge.To)
	}
}

func (g *Graph) AddNode(n Node) {
	g.Nodes = append(g.Nodes, n)
}

func (g *Graph) AddEdge(e Edge) {
	g.Edges = append(g.Edges, e)

	if g.Type == Undirected {
		g.Edges = append(g.Edges, Edge{
			From:      e.To,
			FromAgent: e.ToAgent,
			To:        e.From,
			ToAgent:   e.FromAgent,
		})
	}

	g.AddNode(Node{ID: e.From, Agent: e.FromAgent})
	g.AddNode(Node{ID: e.To, Agent: e.ToAgent})
}

func (g *Graph) GetDestinations(from string) []Node {
	var destinations []Node

	for _, edge := range g.Edges {
		if edge.From == from {
			for _, node := range g.Nodes {
				if node.ID == edge.To {
					destinations = append(destinations, node)
				}
			}
		}
	}

	return destinations
}
