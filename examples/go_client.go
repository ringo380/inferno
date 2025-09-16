package main

/*
Inferno Go Client Example

This example demonstrates how to use the Inferno API with Go.
Includes basic inference, streaming, WebSocket communication, and more.

To run this example:
go mod init inferno-example
go get github.com/gorilla/websocket
go run go_client.go
*/

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"strings"
	"time"

	"github.com/gorilla/websocket"
)

// Client represents the Inferno API client
type Client struct {
	BaseURL    string
	APIKey     string
	HTTPClient *http.Client
}

// NewClient creates a new Inferno client
func NewClient(baseURL, apiKey string) *Client {
	return &Client{
		BaseURL:    strings.TrimSuffix(baseURL, "/"),
		APIKey:     apiKey,
		HTTPClient: &http.Client{Timeout: 30 * time.Second},
	}
}

// Request makes an HTTP request to the Inferno server
func (c *Client) Request(method, endpoint string, body interface{}) (*http.Response, error) {
	var reqBody io.Reader

	if body != nil {
		jsonData, err := json.Marshal(body)
		if err != nil {
			return nil, err
		}
		reqBody = bytes.NewBuffer(jsonData)
	}

	req, err := http.NewRequest(method, c.BaseURL+endpoint, reqBody)
	if err != nil {
		return nil, err
	}

	req.Header.Set("Content-Type", "application/json")
	if c.APIKey != "" {
		req.Header.Set("Authorization", "Bearer "+c.APIKey)
	}

	return c.HTTPClient.Do(req)
}

// Health check structures
type HealthResponse struct {
	Status        string `json:"status"`
	Version       string `json:"version"`
	UptimeSeconds int64  `json:"uptime_seconds"`
	ModelsLoaded  int    `json:"models_loaded"`
}

// Model structures
type ModelInfo struct {
	ID           string   `json:"id"`
	Name         string   `json:"name"`
	Type         string   `json:"type"`
	SizeBytes    int64    `json:"size_bytes"`
	Loaded       bool     `json:"loaded"`
	ContextSize  *int     `json:"context_size,omitempty"`
	Capabilities []string `json:"capabilities"`
}

type ModelsResponse struct {
	Models []ModelInfo `json:"models"`
}

type LoadModelRequest struct {
	GPULayers   *int `json:"gpu_layers,omitempty"`
	ContextSize *int `json:"context_size,omitempty"`
	BatchSize   *int `json:"batch_size,omitempty"`
}

type LoadModelResponse struct {
	Status            string `json:"status"`
	ModelID           string `json:"model_id"`
	MemoryUsageBytes  *int64 `json:"memory_usage_bytes,omitempty"`
	LoadTimeMs        *int64 `json:"load_time_ms,omitempty"`
}

// Inference structures
type InferenceRequest struct {
	Model       string   `json:"model"`
	Prompt      string   `json:"prompt"`
	MaxTokens   int      `json:"max_tokens"`
	Temperature float32  `json:"temperature"`
	TopP        float32  `json:"top_p"`
	TopK        int      `json:"top_k"`
	Stop        []string `json:"stop,omitempty"`
	Stream      bool     `json:"stream"`
}

type Choice struct {
	Text         string  `json:"text"`
	Index        int     `json:"index"`
	FinishReason *string `json:"finish_reason,omitempty"`
}

type Usage struct {
	PromptTokens     int `json:"prompt_tokens"`
	CompletionTokens int `json:"completion_tokens"`
	TotalTokens      int `json:"total_tokens"`
}

type InferenceResponse struct {
	ID               string   `json:"id"`
	Model            string   `json:"model"`
	Choices          []Choice `json:"choices"`
	Usage            *Usage   `json:"usage,omitempty"`
	Created          int64    `json:"created"`
	ProcessingTimeMs *int64   `json:"processing_time_ms,omitempty"`
}

// Embeddings structures
type EmbeddingsRequest struct {
	Model          string   `json:"model"`
	Input          []string `json:"input"`
	EncodingFormat string   `json:"encoding_format"`
}

type EmbeddingData struct {
	Embedding []float32 `json:"embedding"`
	Index     int       `json:"index"`
}

type EmbeddingsResponse struct {
	Model string          `json:"model"`
	Data  []EmbeddingData `json:"data"`
	Usage *Usage          `json:"usage,omitempty"`
}

// Chat completion structures
type ChatMessage struct {
	Role    string `json:"role"`
	Content string `json:"content"`
}

type ChatCompletionRequest struct {
	Model       string        `json:"model"`
	Messages    []ChatMessage `json:"messages"`
	Temperature *float32      `json:"temperature,omitempty"`
	MaxTokens   *int          `json:"max_tokens,omitempty"`
}

type ChatChoice struct {
	Message      ChatMessage `json:"message"`
	Index        int         `json:"index"`
	FinishReason *string     `json:"finish_reason,omitempty"`
}

type ChatCompletionResponse struct {
	Choices []ChatChoice `json:"choices"`
	Usage   *Usage       `json:"usage,omitempty"`
}

// Batch structures
type BatchRequestItem struct {
	ID     string `json:"id"`
	Prompt string `json:"prompt"`
}

type BatchRequest struct {
	Model      string             `json:"model"`
	Requests   []BatchRequestItem `json:"requests"`
	MaxTokens  int                `json:"max_tokens"`
	WebhookURL *string            `json:"webhook_url,omitempty"`
}

type BatchResponse struct {
	BatchID       string `json:"batch_id"`
	Status        string `json:"status"`
	TotalRequests int    `json:"total_requests"`
	Created       int64  `json:"created"`
}

type BatchStatusResponse struct {
	BatchID    string  `json:"batch_id"`
	Status     string  `json:"status"`
	Completed  int     `json:"completed"`
	Failed     int     `json:"failed"`
	Total      int     `json:"total"`
	ResultsURL *string `json:"results_url,omitempty"`
}

// Client methods

// HealthCheck checks the health status of the server
func (c *Client) HealthCheck() (*HealthResponse, error) {
	resp, err := c.Request("GET", "/health", nil)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	var health HealthResponse
	if err := json.NewDecoder(resp.Body).Decode(&health); err != nil {
		return nil, err
	}

	return &health, nil
}

// ListModels lists all available models
func (c *Client) ListModels() ([]ModelInfo, error) {
	resp, err := c.Request("GET", "/models", nil)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	var models ModelsResponse
	if err := json.NewDecoder(resp.Body).Decode(&models); err != nil {
		return nil, err
	}

	return models.Models, nil
}

// LoadModel loads a model into memory
func (c *Client) LoadModel(modelID string, options *LoadModelRequest) (*LoadModelResponse, error) {
	endpoint := fmt.Sprintf("/models/%s/load", modelID)
	resp, err := c.Request("POST", endpoint, options)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	var result LoadModelResponse
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return nil, err
	}

	return &result, nil
}

// UnloadModel unloads a model from memory
func (c *Client) UnloadModel(modelID string) error {
	endpoint := fmt.Sprintf("/models/%s/unload", modelID)
	resp, err := c.Request("POST", endpoint, nil)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	if resp.StatusCode >= 400 {
		return fmt.Errorf("failed to unload model: %s", resp.Status)
	}

	return nil
}

// Inference runs synchronous inference
func (c *Client) Inference(model, prompt string, maxTokens int, temperature float32) (string, error) {
	request := InferenceRequest{
		Model:       model,
		Prompt:      prompt,
		MaxTokens:   maxTokens,
		Temperature: temperature,
		TopP:        0.9,
		TopK:        40,
		Stream:      false,
	}

	resp, err := c.Request("POST", "/inference", request)
	if err != nil {
		return "", err
	}
	defer resp.Body.Close()

	var result InferenceResponse
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return "", err
	}

	if len(result.Choices) == 0 {
		return "", fmt.Errorf("no response received")
	}

	return result.Choices[0].Text, nil
}

// Embeddings generates embeddings for text inputs
func (c *Client) Embeddings(model string, texts []string) ([][]float32, error) {
	request := EmbeddingsRequest{
		Model:          model,
		Input:          texts,
		EncodingFormat: "float",
	}

	resp, err := c.Request("POST", "/embeddings", request)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	var result EmbeddingsResponse
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return nil, err
	}

	embeddings := make([][]float32, len(result.Data))
	for i, data := range result.Data {
		embeddings[i] = data.Embedding
	}

	return embeddings, nil
}

// ChatCompletion performs OpenAI-compatible chat completion
func (c *Client) ChatCompletion(model string, messages []ChatMessage) (string, error) {
	temperature := float32(0.7)
	maxTokens := 100

	request := ChatCompletionRequest{
		Model:       model,
		Messages:    messages,
		Temperature: &temperature,
		MaxTokens:   &maxTokens,
	}

	resp, err := c.Request("POST", "/v1/chat/completions", request)
	if err != nil {
		return "", err
	}
	defer resp.Body.Close()

	var result ChatCompletionResponse
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return "", err
	}

	if len(result.Choices) == 0 {
		return "", fmt.Errorf("no response received")
	}

	return result.Choices[0].Message.Content, nil
}

// BatchInference submits a batch of prompts for processing
func (c *Client) BatchInference(model string, prompts []string) (string, error) {
	requests := make([]BatchRequestItem, len(prompts))
	for i, prompt := range prompts {
		requests[i] = BatchRequestItem{
			ID:     fmt.Sprintf("req_%d", i),
			Prompt: prompt,
		}
	}

	request := BatchRequest{
		Model:     model,
		Requests:  requests,
		MaxTokens: 100,
	}

	resp, err := c.Request("POST", "/batch", request)
	if err != nil {
		return "", err
	}
	defer resp.Body.Close()

	var result BatchResponse
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return "", err
	}

	return result.BatchID, nil
}

// GetBatchStatus gets the status of a batch job
func (c *Client) GetBatchStatus(batchID string) (*BatchStatusResponse, error) {
	endpoint := fmt.Sprintf("/batch/%s", batchID)
	resp, err := c.Request("GET", endpoint, nil)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	var result BatchStatusResponse
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return nil, err
	}

	return &result, nil
}

// WebSocket client
type WebSocketClient struct {
	URL    string
	APIKey string
	conn   *websocket.Conn
}

// NewWebSocketClient creates a new WebSocket client
func NewWebSocketClient(wsURL, apiKey string) *WebSocketClient {
	return &WebSocketClient{
		URL:    wsURL,
		APIKey: apiKey,
	}
}

// Connect connects to the WebSocket server
func (ws *WebSocketClient) Connect() error {
	u, err := url.Parse(ws.URL)
	if err != nil {
		return err
	}

	conn, _, err := websocket.DefaultDialer.Dial(u.String(), nil)
	if err != nil {
		return err
	}

	ws.conn = conn
	fmt.Println("WebSocket connection opened")

	// Send authentication if API key provided
	if ws.APIKey != "" {
		authMsg := map[string]interface{}{
			"type":  "auth",
			"token": ws.APIKey,
		}

		if err := conn.WriteJSON(authMsg); err != nil {
			return err
		}
	}

	return nil
}

// SendInference sends an inference request
func (ws *WebSocketClient) SendInference(model, prompt string, maxTokens int) error {
	if ws.conn == nil {
		return fmt.Errorf("WebSocket not connected")
	}

	request := map[string]interface{}{
		"type":       "inference",
		"id":         fmt.Sprintf("req_%d", time.Now().UnixMilli()),
		"model":      model,
		"prompt":     prompt,
		"max_tokens": maxTokens,
		"stream":     true,
	}

	return ws.conn.WriteJSON(request)
}

// Listen listens for WebSocket messages
func (ws *WebSocketClient) Listen() error {
	if ws.conn == nil {
		return fmt.Errorf("WebSocket not connected")
	}

	for {
		var message map[string]interface{}
		err := ws.conn.ReadJSON(&message)
		if err != nil {
			return err
		}

		switch message["type"] {
		case "token":
			if token, ok := message["token"].(string); ok {
				fmt.Print(token)
			}
		case "complete":
			fmt.Println("\n[Inference complete]")
			return nil
		case "error":
			if errorMsg, ok := message["message"].(string); ok {
				fmt.Printf("\n[Error: %s]", errorMsg)
			}
			return nil
		}
	}
}

// Close closes the WebSocket connection
func (ws *WebSocketClient) Close() error {
	if ws.conn != nil {
		return ws.conn.Close()
	}
	return nil
}

func main() {
	fmt.Println("=== Inferno Go Client Example ===\n")

	// Initialize client
	client := NewClient("http://localhost:8080", "your_api_key_here")

	// 1. Health check
	fmt.Println("1. Health Check")
	if health, err := client.HealthCheck(); err != nil {
		fmt.Printf("   Error: %v\n", err)
	} else {
		fmt.Printf("   Status: %s\n", health.Status)
		fmt.Printf("   Version: %s\n\n", health.Version)
	}

	// 2. List models
	fmt.Println("2. Available Models")
	if models, err := client.ListModels(); err != nil {
		fmt.Printf("   Error: %v\n", err)
	} else {
		for _, model := range models {
			fmt.Printf("   - %s: %s (%s)\n", model.ID, model.Name, model.Type)
		}
		fmt.Println()
	}

	// 3. Load a model
	fmt.Println("3. Loading Model")
	modelID := "llama-2-7b"
	// Uncomment to actually load:
	// if result, err := client.LoadModel(modelID, nil); err != nil {
	//     fmt.Printf("   Error: %v\n", err)
	// } else {
	//     fmt.Printf("   Model loaded: %s\n\n", result.Status)
	// }

	// 4. Simple inference
	fmt.Println("4. Simple Inference")
	prompt := "What is artificial intelligence?"
	fmt.Printf("   Prompt: %s\n", prompt)
	// Uncomment to run inference:
	// if response, err := client.Inference(modelID, prompt, 50, 0.7); err != nil {
	//     fmt.Printf("   Error: %v\n", err)
	// } else {
	//     fmt.Printf("   Response: %s\n\n", response)
	// }

	// 5. Generate embeddings
	fmt.Println("5. Text Embeddings")
	texts := []string{"Hello world", "How are you?", "Machine learning is fascinating"}
	fmt.Printf("   Texts: %v\n", texts)
	// Uncomment to generate embeddings:
	// if embeddings, err := client.Embeddings(modelID, texts); err != nil {
	//     fmt.Printf("   Error: %v\n", err)
	// } else {
	//     fmt.Printf("   Generated %d embeddings\n", len(embeddings))
	//     if len(embeddings) > 0 {
	//         fmt.Printf("   Embedding dimension: %d\n\n", len(embeddings[0]))
	//     }
	// }

	// 6. Chat completion (OpenAI compatible)
	fmt.Println("6. Chat Completion")
	messages := []ChatMessage{
		{Role: "system", Content: "You are a helpful assistant."},
		{Role: "user", Content: "What is the capital of France?"},
	}
	fmt.Printf("   Messages: %d\n", len(messages))
	// Uncomment to run chat:
	// if response, err := client.ChatCompletion(modelID, messages); err != nil {
	//     fmt.Printf("   Error: %v\n", err)
	// } else {
	//     fmt.Printf("   Assistant: %s\n\n", response)
	// }

	// 7. Batch processing
	fmt.Println("7. Batch Processing")
	prompts := []string{
		"What is Python?",
		"Explain quantum computing",
		"How does photosynthesis work?",
	}
	fmt.Printf("   Batch size: %d\n", len(prompts))
	// Uncomment to submit batch:
	// if batchID, err := client.BatchInference(modelID, prompts); err != nil {
	//     fmt.Printf("   Error: %v\n", err)
	// } else {
	//     fmt.Printf("   Batch ID: %s\n", batchID)
	//
	//     // Wait for completion
	//     for {
	//         if status, err := client.GetBatchStatus(batchID); err != nil {
	//             fmt.Printf("   Status check error: %v\n", err)
	//             break
	//         } else if status.Status == "completed" {
	//             fmt.Printf("   Completed: %d responses\n\n", status.Completed)
	//             break
	//         }
	//         time.Sleep(1 * time.Second)
	//     }
	// }

	// 8. WebSocket streaming (uncomment to test)
	fmt.Println("8. WebSocket Streaming")
	fmt.Println("   Setting up WebSocket client...")
	// wsClient := NewWebSocketClient("ws://localhost:8080/ws", "your_api_key_here")
	// if err := wsClient.Connect(); err != nil {
	//     fmt.Printf("   Connection error: %v\n", err)
	// } else {
	//     fmt.Println("   Sending inference request...")
	//     if err := wsClient.SendInference(modelID, "Tell me a joke", 50); err != nil {
	//         fmt.Printf("   Send error: %v\n", err)
	//     } else {
	//         fmt.Print("   Response: ")
	//         if err := wsClient.Listen(); err != nil {
	//             fmt.Printf("   Listen error: %v\n", err)
	//         }
	//     }
	//     wsClient.Close()
	// }

	fmt.Println("\n=== Example Complete ===")
}