package main

import (
	"context"
	"fmt"
	"os"

	"github.com/ElioenaiFerrari/grace/internal/pipeline"
	"github.com/joho/godotenv"
)

func init() {
	godotenv.Load()
}

func main() {
	ctx := context.Background()
	apiKey := os.Getenv("OPENAI_API_KEY")

	developer := pipeline.NewAgent(apiKey, "developer")
	productManager := pipeline.NewAgent(apiKey, "product manager")
	customer := pipeline.NewAgent(apiKey, "customer")

	developer.SetSystemMessage(ctx, "Olá, eu sou um desenvolvedor e irei implementar o projeto")
	productManager.SetSystemMessage(ctx, "Olá, eu sou um gerente de produto e irei definir o escopo do projeto")
	customer.SetSystemMessage(ctx, "Olá, eu sou um cliente e quero um jogo de cobrinha escrito em Golang")

	room := pipeline.NewRoom()

	room.AddConnection(customer, productManager)
	room.AddConnection(productManager, developer)
	room.AddConnection(developer, productManager)

	ch, err := room.Run(ctx)
	if err != nil {
		panic(err)
	}

	f, err := os.Create("trace.out")
	if err != nil {
		panic(err)
	}

	for message := range ch {
		f.WriteString(message.Content + "\n")
		fmt.Println(message.Content)
	}
}
