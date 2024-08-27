FROM golang:1.22-alpine as builder

# Set the Current Working Directory inside the container
WORKDIR /app

# Copy go mod and sum files
COPY go.mod go.sum ./

# Download all dependencies. Dependencies will be cached if the go.mod and go.sum files are not changed
RUN go mod tidy -x

# Copy the source from the current directory to the Working Directory inside the container
COPY . .

# Build the Go app
RUN GOOS=linux GOARCH=amd64 go build -ldflags="-w -s" -o /go/bin/app ./cmd/grace

# Start a new stage from scratch
# distroless
FROM gcr.io/distroless/base

# Copy the Pre-built binary file from the previous stage
COPY --from=builder /go/bin/app /go/bin/app

# Command to run the executable
CMD ["/go/bin/app"]
