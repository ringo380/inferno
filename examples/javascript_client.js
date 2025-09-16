#!/usr/bin/env node
/**
 * Inferno JavaScript/Node.js Client Example
 *
 * This example demonstrates how to use the Inferno API with JavaScript.
 * Includes basic inference, streaming, WebSocket communication, and more.
 */

const fetch = require('node-fetch');
const WebSocket = require('ws');
const EventSource = require('eventsource');

class InfernoClient {
    /**
     * Initialize the Inferno client.
     *
     * @param {string} baseUrl - The base URL of the Inferno server
     * @param {string} apiKey - Optional API key for authentication
     */
    constructor(baseUrl = 'http://localhost:8080', apiKey = null) {
        this.baseUrl = baseUrl.replace(/\/$/, '');
        this.apiKey = apiKey;
        this.headers = {
            'Content-Type': 'application/json'
        };

        if (apiKey) {
            this.headers['Authorization'] = `Bearer ${apiKey}`;
        }
    }

    /**
     * Make an HTTP request to the Inferno server.
     */
    async request(endpoint, method = 'GET', body = null) {
        const url = `${this.baseUrl}${endpoint}`;
        const options = {
            method,
            headers: this.headers
        };

        if (body) {
            options.body = JSON.stringify(body);
        }

        const response = await fetch(url, options);

        if (!response.ok) {
            const error = await response.json().catch(() => ({
                error: { message: response.statusText }
            }));
            throw new Error(`HTTP ${response.status}: ${error.error?.message || 'Unknown error'}`);
        }

        return response.json();
    }

    /**
     * Check the health status of the server.
     */
    async healthCheck() {
        return this.request('/health');
    }

    /**
     * List all available models.
     */
    async listModels() {
        const response = await this.request('/models');
        return response.models;
    }

    /**
     * Load a model into memory.
     */
    async loadModel(modelId, options = {}) {
        return this.request(`/models/${modelId}/load`, 'POST', options);
    }

    /**
     * Unload a model from memory.
     */
    async unloadModel(modelId) {
        return this.request(`/models/${modelId}/unload`, 'POST');
    }

    /**
     * Run synchronous inference.
     */
    async inference(model, prompt, options = {}) {
        const requestData = {
            model,
            prompt,
            max_tokens: 100,
            temperature: 0.7,
            top_p: 0.9,
            top_k: 40,
            stream: false,
            ...options
        };

        const response = await this.request('/inference', 'POST', requestData);
        return response.choices[0].text;
    }

    /**
     * Stream inference results using Server-Sent Events.
     */
    async streamInference(model, prompt, options = {}) {
        const requestData = {
            model,
            prompt,
            max_tokens: 100,
            stream: true,
            ...options
        };

        // Create a POST request with streaming
        const response = await fetch(`${this.baseUrl}/inference/stream`, {
            method: 'POST',
            headers: {
                ...this.headers,
                'Accept': 'text/event-stream'
            },
            body: JSON.stringify(requestData)
        });

        if (!response.ok) {
            throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }

        return new Promise((resolve, reject) => {
            const eventSource = new EventSource(`${this.baseUrl}/inference/stream`, {
                headers: this.headers
            });

            let fullText = '';

            eventSource.onmessage = (event) => {
                try {
                    const data = JSON.parse(event.data);

                    if (data.token) {
                        process.stdout.write(data.token);
                        fullText += data.token;
                    } else if (data.done) {
                        eventSource.close();
                        resolve(fullText);
                    } else if (data.error) {
                        eventSource.close();
                        reject(new Error(data.error));
                    }
                } catch (error) {
                    eventSource.close();
                    reject(error);
                }
            };

            eventSource.onerror = (error) => {
                eventSource.close();
                reject(error);
            };
        });
    }

    /**
     * Generate embeddings for text inputs.
     */
    async embeddings(model, texts) {
        const requestData = {
            model,
            input: texts,
            encoding_format: 'float'
        };

        const response = await this.request('/embeddings', 'POST', requestData);
        return response.data.map(item => item.embedding);
    }

    /**
     * OpenAI-compatible chat completion.
     */
    async chatCompletion(model, messages, options = {}) {
        const requestData = {
            model,
            messages,
            ...options
        };

        const response = await this.request('/v1/chat/completions', 'POST', requestData);
        return response.choices[0].message.content;
    }

    /**
     * Submit a batch of prompts for processing.
     */
    async batchInference(model, prompts, options = {}) {
        const requests = prompts.map((prompt, i) => ({
            id: `req_${i}`,
            prompt: prompt
        }));

        const requestData = {
            model,
            requests,
            max_tokens: 100,
            ...options
        };

        const response = await this.request('/batch', 'POST', requestData);
        return response.batch_id;
    }

    /**
     * Get the status of a batch job.
     */
    async getBatchStatus(batchId) {
        return this.request(`/batch/${batchId}`);
    }

    /**
     * Get the results of a completed batch job.
     */
    async getBatchResults(batchId) {
        const response = await this.request(`/batch/${batchId}/results`);
        return response.results;
    }
}

class InfernoWebSocketClient {
    /**
     * WebSocket client for real-time streaming with Inferno.
     */
    constructor(url = 'ws://localhost:8080/ws', apiKey = null) {
        this.url = url;
        this.apiKey = apiKey;
        this.ws = null;
        this.isConnected = false;
    }

    /**
     * Connect to the WebSocket server.
     */
    connect() {
        return new Promise((resolve, reject) => {
            this.ws = new WebSocket(this.url);

            this.ws.on('open', () => {
                console.log('WebSocket connection opened');
                this.isConnected = true;

                // Send authentication if API key provided
                if (this.apiKey) {
                    this.ws.send(JSON.stringify({
                        type: 'auth',
                        token: this.apiKey
                    }));
                }

                resolve();
            });

            this.ws.on('message', (data) => {
                const message = JSON.parse(data.toString());
                this.handleMessage(message);
            });

            this.ws.on('error', (error) => {
                console.error('WebSocket error:', error);
                reject(error);
            });

            this.ws.on('close', (code, reason) => {
                console.log(`WebSocket connection closed: ${reason}`);
                this.isConnected = false;
            });
        });
    }

    /**
     * Handle incoming WebSocket messages.
     */
    handleMessage(message) {
        switch (message.type) {
            case 'token':
                process.stdout.write(message.token);
                break;
            case 'complete':
                console.log('\n[Inference complete]');
                break;
            case 'error':
                console.log(`\n[Error: ${message.message}]`);
                break;
            default:
                console.log('Unknown message type:', message.type);
        }
    }

    /**
     * Send an inference request.
     */
    sendInference(model, prompt, options = {}) {
        if (!this.isConnected) {
            throw new Error('WebSocket not connected');
        }

        const request = {
            type: 'inference',
            id: `req_${Date.now()}`,
            model: model,
            prompt: prompt,
            max_tokens: 100,
            stream: true,
            ...options
        };

        this.ws.send(JSON.stringify(request));
    }

    /**
     * Close the WebSocket connection.
     */
    close() {
        if (this.ws) {
            this.ws.close();
        }
    }
}

/**
 * Example usage and demonstrations.
 */
async function main() {
    console.log('=== Inferno JavaScript Client Example ===\n');

    // Initialize client
    const client = new InfernoClient('http://localhost:8080', 'your_api_key_here');

    try {
        // 1. Health check
        console.log('1. Health Check');
        const health = await client.healthCheck();
        console.log(`   Status: ${health.status}`);
        console.log(`   Version: ${health.version}\n`);

        // 2. List models
        console.log('2. Available Models');
        const models = await client.listModels();
        models.forEach(model => {
            console.log(`   - ${model.id}: ${model.name} (${model.type})`);
        });
        console.log();

        // 3. Load a model
        console.log('3. Loading Model');
        const modelId = 'llama-2-7b';
        // Uncomment to actually load:
        // const loadResult = await client.loadModel(modelId, { gpu_layers: 32 });
        // console.log(`   Model loaded: ${loadResult.status}\n`);

        // 4. Simple inference
        console.log('4. Simple Inference');
        const prompt = 'What is artificial intelligence?';
        console.log(`   Prompt: ${prompt}`);
        // Uncomment to run inference:
        // const response = await client.inference(modelId, prompt, { max_tokens: 50 });
        // console.log(`   Response: ${response}\n`);

        // 5. Streaming inference
        console.log('5. Streaming Inference');
        const streamPrompt = 'Tell me a short story about a robot';
        console.log(`   Prompt: ${streamPrompt}`);
        console.log('   Response: ');
        // Uncomment to stream:
        // await client.streamInference(modelId, streamPrompt, { max_tokens: 100 });
        // console.log('\n');

        // 6. Generate embeddings
        console.log('6. Text Embeddings');
        const texts = ['Hello world', 'How are you?', 'Machine learning is fascinating'];
        console.log(`   Texts: ${JSON.stringify(texts)}`);
        // Uncomment to generate embeddings:
        // const embeddings = await client.embeddings(modelId, texts);
        // console.log(`   Generated ${embeddings.length} embeddings`);
        // console.log(`   Embedding dimension: ${embeddings[0].length}\n`);

        // 7. Chat completion (OpenAI compatible)
        console.log('7. Chat Completion');
        const messages = [
            { role: 'system', content: 'You are a helpful assistant.' },
            { role: 'user', content: 'What is the capital of France?' }
        ];
        console.log(`   Messages: ${messages.length}`);
        // Uncomment to run chat:
        // const chatResponse = await client.chatCompletion(modelId, messages);
        // console.log(`   Assistant: ${chatResponse}\n`);

        // 8. Batch processing
        console.log('8. Batch Processing');
        const prompts = [
            'What is Python?',
            'Explain quantum computing',
            'How does photosynthesis work?'
        ];
        console.log(`   Batch size: ${prompts.length}`);
        // Uncomment to submit batch:
        // const batchId = await client.batchInference(modelId, prompts);
        // console.log(`   Batch ID: ${batchId}`);
        //
        // // Wait for completion
        // while (true) {
        //     const status = await client.getBatchStatus(batchId);
        //     if (status.status === 'completed') {
        //         break;
        //     }
        //     await new Promise(resolve => setTimeout(resolve, 1000));
        // }
        //
        // const results = await client.getBatchResults(batchId);
        // console.log(`   Completed: ${results.length} responses\n`);

        // 9. WebSocket streaming (uncomment to test)
        console.log('9. WebSocket Streaming');
        console.log('   Setting up WebSocket client...');
        // const wsClient = new InfernoWebSocketClient('ws://localhost:8080/ws', 'your_api_key_here');
        // await wsClient.connect();
        // console.log('   Sending inference request...');
        // wsClient.sendInference(modelId, 'Tell me a joke', { max_tokens: 50 });
        //
        // // Keep connection alive for a bit
        // setTimeout(() => {
        //     wsClient.close();
        // }, 10000);

        console.log('\n=== Example Complete ===');

    } catch (error) {
        console.error('Error:', error.message);
    }
}

// Utility functions for browser usage
if (typeof window !== 'undefined') {
    // Browser environment
    window.InfernoClient = InfernoClient;
    window.InfernoWebSocketClient = InfernoWebSocketClient;
} else {
    // Node.js environment
    if (require.main === module) {
        main().catch(console.error);
    }

    module.exports = {
        InfernoClient,
        InfernoWebSocketClient
    };
}