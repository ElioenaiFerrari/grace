package main

import (
	"context"
	"log"
	"os"

	"github.com/ElioenaiFerrari/grace/internal/pipeline"
	"github.com/gofiber/fiber/v2"
	"github.com/gofiber/websocket/v2"
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

	room.Print()

	app := fiber.New()

	app.Use("/ws", func(c *fiber.Ctx) error {
		if websocket.IsWebSocketUpgrade(c) {
			return c.Next()
		}

		return c.SendStatus(fiber.StatusUpgradeRequired)
	})

	app.Get("/ws/:id", websocket.New(func(c *websocket.Conn) {
		log.Println("connected", c.Params("id"))
		ch, err := room.Run(ctx)
		if err != nil {
			panic(err)
		}

		for message := range ch {
			if err := c.WriteJSON(message); err != nil {
				return
			}
		}
	}))

	app.Listen(":4000")
}
