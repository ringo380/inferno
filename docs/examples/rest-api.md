# 🌐 REST API Examples

Complete examples for integrating with Inferno's REST API across multiple programming languages and use cases.

## Overview

This guide provides practical examples for:
- ✅ **Basic API Usage** - Authentication, models, and simple requests
- ✅ **Text Generation** - Completions and chat interactions
- ✅ **Streaming Responses** - Real-time text generation
- ✅ **Embeddings** - Semantic search and similarity
- ✅ **Batch Processing** - Handle multiple requests efficiently
- ✅ **Error Handling** - Robust retry logic and fallbacks
- ✅ **Production Patterns** - Scalable integration strategies

## API Fundamentals

### Base Configuration

```bash
# Default Inferno server
BASE_URL="http://localhost:8080"
API_VERSION="/v1"
FULL_URL="${BASE_URL}${API_VERSION}"

# With authentication (if enabled)
API_KEY="your-api-key"
```

### Health Check

```bash
# Test server connectivity
curl ${BASE_URL}/health

# Expected response:
# {
#   "status": "healthy",
#   "version": "1.0.0",
#   "models_loaded": ["gpt2", "microsoft/DialoGPT-medium"],
#   "uptime": "2h 15m 30s"
# }
```

## cURL Examples

### Basic Operations

```bash
# List available models
curl -X GET ${FULL_URL}/models \
  -H "Authorization: Bearer ${API_KEY}"

# Get specific model info
curl -X GET ${FULL_URL}/models/gpt2 \
  -H "Authorization: Bearer ${API_KEY}"

# Load a model into memory
curl -X POST ${FULL_URL}/models/gpt2/load \
  -H "Authorization: Bearer ${API_KEY}"

# Check model status
curl -X GET ${FULL_URL}/models/gpt2/status \
  -H "Authorization: Bearer ${API_KEY}"
```

### Text Completions

```bash
# Basic text completion
curl -X POST ${FULL_URL}/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ${API_KEY}" \
  -d '{
    "model": "gpt2",
    "prompt": "The future of artificial intelligence is",
    "max_tokens": 100,
    "temperature": 0.7
  }'

# Advanced completion with parameters
curl -X POST ${FULL_URL}/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ${API_KEY}" \
  -d '{
    "model": "gpt2",
    "prompt": "Write a Python function to calculate fibonacci numbers:",
    "max_tokens": 200,
    "temperature": 0.2,
    "top_p": 0.9,
    "frequency_penalty": 0.1,
    "presence_penalty": 0.1,
    "stop": ["\n\n"]
  }'
```

### Chat Completions

```bash
# Simple chat
curl -X POST ${FULL_URL}/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ${API_KEY}" \
  -d '{
    "model": "microsoft/DialoGPT-medium",
    "messages": [
      {"role": "user", "content": "Hello! How are you?"}
    ],
    "max_tokens": 150
  }'

# Conversation with history
curl -X POST ${FULL_URL}/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ${API_KEY}" \
  -d '{
    "model": "microsoft/DialoGPT-medium",
    "messages": [
      {"role": "system", "content": "You are a helpful AI assistant."},
      {"role": "user", "content": "What is machine learning?"},
      {"role": "assistant", "content": "Machine learning is a subset of artificial intelligence that enables computers to learn and improve from experience without being explicitly programmed."},
      {"role": "user", "content": "Can you give me an example?"}
    ],
    "max_tokens": 200,
    "temperature": 0.8
  }'
```

### Streaming Responses

```bash
# Streaming completion
curl -X POST ${FULL_URL}/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ${API_KEY}" \
  -d '{
    "model": "gpt2",
    "prompt": "Write a story about a robot:",
    "max_tokens": 200,
    "stream": true
  }' --no-buffer

# Streaming chat
curl -X POST ${FULL_URL}/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ${API_KEY}" \
  -d '{
    "model": "microsoft/DialoGPT-medium",
    "messages": [
      {"role": "user", "content": "Tell me a joke"}
    ],
    "stream": true
  }' --no-buffer
```

### Embeddings

```bash
# Get embeddings for text
curl -X POST ${FULL_URL}/embeddings \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ${API_KEY}" \
  -d '{
    "model": "text-embedding-ada-002",
    "input": ["Hello world", "How are you?"]
  }'

# Single text embedding
curl -X POST ${FULL_URL}/embeddings \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ${API_KEY}" \
  -d '{
    "model": "text-embedding-ada-002",
    "input": "This is a sample text for embedding"
  }'
```

## Python Examples

### Basic Client Setup

```python
import requests
import json
import time
from typing import List, Dict, Optional, Generator

class InfernoClient:
    """Simple Inferno REST API client."""

    def __init__(self, base_url: str = "http://localhost:8080", api_key: str = "not-needed"):
        self.base_url = base_url.rstrip("/")
        self.api_key = api_key
        self.session = requests.Session()

        # Set default headers
        self.session.headers.update({
            "Content-Type": "application/json",
            "Authorization": f"Bearer {api_key}" if api_key != "not-needed" else ""
        })

    def _make_request(self, method: str, endpoint: str, **kwargs) -> requests.Response:
        """Make HTTP request with error handling."""
        url = f"{self.base_url}/v1{endpoint}"

        try:
            response = self.session.request(method, url, **kwargs)
            response.raise_for_status()
            return response
        except requests.exceptions.RequestException as e:
            print(f"Request failed: {e}")
            raise

    def health_check(self) -> Dict:
        """Check server health."""
        response = self.session.get(f"{self.base_url}/health")
        return response.json()

    def list_models(self) -> List[str]:
        """List available models."""
        response = self._make_request("GET", "/models")
        data = response.json()
        return [model["id"] for model in data["data"]]

    def get_model_info(self, model_id: str) -> Dict:
        """Get information about a specific model."""
        response = self._make_request("GET", f"/models/{model_id}")
        return response.json()

    def load_model(self, model_id: str) -> Dict:
        """Load a model into memory."""
        response = self._make_request("POST", f"/models/{model_id}/load")
        return response.json()

    def completion(
        self,
        prompt: str,
        model: str = "gpt2",
        max_tokens: int = 100,
        temperature: float = 0.7,
        **kwargs
    ) -> Dict:
        """Generate text completion."""
        data = {
            "model": model,
            "prompt": prompt,
            "max_tokens": max_tokens,
            "temperature": temperature,
            **kwargs
        }

        response = self._make_request("POST", "/completions", json=data)
        return response.json()

    def chat_completion(
        self,
        messages: List[Dict[str, str]],
        model: str = "microsoft/DialoGPT-medium",
        max_tokens: int = 150,
        temperature: float = 0.7,
        **kwargs
    ) -> Dict:
        """Generate chat completion."""
        data = {
            "model": model,
            "messages": messages,
            "max_tokens": max_tokens,
            "temperature": temperature,
            **kwargs
        }

        response = self._make_request("POST", "/chat/completions", json=data)
        return response.json()

    def embeddings(
        self,
        input_text: List[str] | str,
        model: str = "text-embedding-ada-002"
    ) -> Dict:
        """Get embeddings for text."""
        if isinstance(input_text, str):
            input_text = [input_text]

        data = {
            "model": model,
            "input": input_text
        }

        response = self._make_request("POST", "/embeddings", json=data)
        return response.json()

    def stream_completion(
        self,
        prompt: str,
        model: str = "gpt2",
        max_tokens: int = 200,
        temperature: float = 0.7,
        **kwargs
    ) -> Generator[str, None, None]:
        """Stream text completion."""
        data = {
            "model": model,
            "prompt": prompt,
            "max_tokens": max_tokens,
            "temperature": temperature,
            "stream": True,
            **kwargs
        }

        url = f"{self.base_url}/v1/completions"

        with self.session.post(url, json=data, stream=True) as response:
            response.raise_for_status()

            for line in response.iter_lines():
                if line:
                    line = line.decode('utf-8')
                    if line.startswith('data: '):
                        data_str = line[6:]  # Remove 'data: ' prefix

                        if data_str.strip() == '[DONE]':
                            break

                        try:
                            chunk_data = json.loads(data_str)
                            if chunk_data.get('choices') and chunk_data['choices'][0].get('text'):
                                yield chunk_data['choices'][0]['text']
                        except json.JSONDecodeError:
                            continue

# Example usage
client = InfernoClient()

# Check health
try:
    health = client.health_check()
    print(f"Server status: {health['status']}")
except Exception as e:
    print(f"Server not available: {e}")

# List models
models = client.list_models()
print(f"Available models: {models}")

# Generate text
response = client.completion(
    prompt="The benefits of renewable energy include",
    model="gpt2",
    max_tokens=100
)
print(f"Generated text: {response['choices'][0]['text']}")

# Chat example
chat_response = client.chat_completion(
    messages=[
        {"role": "user", "content": "What is the capital of France?"}
    ],
    model="microsoft/DialoGPT-medium"
)
print(f"Chat response: {chat_response['choices'][0]['message']['content']}")

# Streaming example
print("Streaming response:")
for chunk in client.stream_completion("Write a short story about AI:"):
    print(chunk, end="", flush=True)
print()  # New line after streaming
```

### Advanced Python Examples

```python
import asyncio
import aiohttp
import numpy as np
from sklearn.metrics.pairwise import cosine_similarity
from typing import List, Dict, Optional, Tuple

class AsyncInfernoClient:
    """Async Inferno client for high-performance applications."""

    def __init__(self, base_url: str = "http://localhost:8080", api_key: str = "not-needed"):
        self.base_url = base_url.rstrip("/")
        self.api_key = api_key
        self.session = None

    async def __aenter__(self):
        self.session = aiohttp.ClientSession(
            headers={
                "Content-Type": "application/json",
                "Authorization": f"Bearer {self.api_key}" if self.api_key != "not-needed" else ""
            }
        )
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        if self.session:
            await self.session.close()

    async def completion(self, prompt: str, model: str = "gpt2", **kwargs) -> Dict:
        """Async text completion."""
        data = {
            "model": model,
            "prompt": prompt,
            **kwargs
        }

        async with self.session.post(f"{self.base_url}/v1/completions", json=data) as response:
            response.raise_for_status()
            return await response.json()

    async def batch_completions(self, prompts: List[str], model: str = "gpt2", **kwargs) -> List[Dict]:
        """Process multiple completions concurrently."""
        tasks = [self.completion(prompt, model, **kwargs) for prompt in prompts]
        results = await asyncio.gather(*tasks, return_exceptions=True)

        # Separate successful results from exceptions
        successful_results = []
        for i, result in enumerate(results):
            if isinstance(result, Exception):
                print(f"Prompt {i} failed: {result}")
                successful_results.append(None)
            else:
                successful_results.append(result)

        return successful_results

    async def embeddings(self, texts: List[str], model: str = "text-embedding-ada-002") -> List[List[float]]:
        """Get embeddings for multiple texts."""
        data = {
            "model": model,
            "input": texts
        }

        async with self.session.post(f"{self.base_url}/v1/embeddings", json=data) as response:
            response.raise_for_status()
            result = await response.json()
            return [item["embedding"] for item in result["data"]]

class SemanticSearch:
    """Semantic search using Inferno embeddings."""

    def __init__(self, client: InfernoClient):
        self.client = client
        self.documents = []
        self.embeddings = []

    def add_documents(self, documents: List[str]):
        """Add documents to the search index."""
        self.documents.extend(documents)

        # Get embeddings for new documents
        response = self.client.embeddings(documents)
        new_embeddings = [item["embedding"] for item in response["data"]]
        self.embeddings.extend(new_embeddings)

    def search(self, query: str, top_k: int = 5) -> List[Tuple[str, float]]:
        """Search for similar documents."""
        if not self.documents:
            return []

        # Get query embedding
        query_response = self.client.embeddings([query])
        query_embedding = np.array(query_response["data"][0]["embedding"]).reshape(1, -1)

        # Calculate similarities
        doc_embeddings = np.array(self.embeddings)
        similarities = cosine_similarity(query_embedding, doc_embeddings)[0]

        # Get top-k results
        top_indices = np.argsort(similarities)[::-1][:top_k]

        results = []
        for idx in top_indices:
            results.append((self.documents[idx], similarities[idx]))

        return results

# Example usage with semantic search
client = InfernoClient()

# Create semantic search index
search = SemanticSearch(client)

# Add documents
documents = [
    "Machine learning is a subset of artificial intelligence",
    "Deep learning uses neural networks with multiple layers",
    "Natural language processing helps computers understand human language",
    "Computer vision enables machines to interpret visual information",
    "Reinforcement learning teaches agents through rewards and penalties"
]

search.add_documents(documents)

# Search for similar documents
query = "What is AI?"
results = search.search(query, top_k=3)

print(f"Query: {query}")
for doc, similarity in results:
    print(f"Similarity: {similarity:.3f} - {doc}")
```

### Error Handling and Retry Logic

```python
import time
import random
from functools import wraps
from typing import Callable, Any

def retry_with_backoff(
    max_retries: int = 3,
    initial_delay: float = 1.0,
    max_delay: float = 60.0,
    exponential_base: float = 2.0,
    jitter: bool = True
):
    """Decorator for retrying functions with exponential backoff."""
    def decorator(func: Callable) -> Callable:
        @wraps(func)
        def wrapper(*args, **kwargs) -> Any:
            delay = initial_delay

            for attempt in range(max_retries):
                try:
                    return func(*args, **kwargs)
                except Exception as e:
                    if attempt == max_retries - 1:
                        raise e

                    # Calculate delay with jitter
                    if jitter:
                        delay_with_jitter = delay * (0.5 + random.random() * 0.5)
                    else:
                        delay_with_jitter = delay

                    actual_delay = min(delay_with_jitter, max_delay)

                    print(f"Attempt {attempt + 1} failed: {e}. Retrying in {actual_delay:.2f}s...")
                    time.sleep(actual_delay)

                    delay *= exponential_base

            return None
        return wrapper
    return decorator

class RobustInfernoClient(InfernoClient):
    """Inferno client with robust error handling."""

    @retry_with_backoff(max_retries=3)
    def robust_completion(self, prompt: str, model: str = "gpt2", **kwargs) -> Optional[Dict]:
        """Completion with retry logic."""
        return self.completion(prompt, model, **kwargs)

    def completion_with_fallback(
        self,
        prompt: str,
        models: List[str] = ["gpt2", "distilgpt2"],
        **kwargs
    ) -> Optional[Dict]:
        """Try multiple models in order until one succeeds."""
        for model in models:
            try:
                return self.robust_completion(prompt, model, **kwargs)
            except Exception as e:
                print(f"Model {model} failed: {e}")
                continue

        print("All models failed")
        return None

    def batch_process_with_fallback(
        self,
        prompts: List[str],
        model: str = "gpt2",
        fallback_model: str = "distilgpt2"
    ) -> List[Dict]:
        """Process batch with fallback for individual failures."""
        results = []

        for i, prompt in enumerate(prompts):
            try:
                result = self.robust_completion(prompt, model)
                if result:
                    results.append({
                        "index": i,
                        "prompt": prompt,
                        "response": result,
                        "model_used": model,
                        "error": None
                    })
                else:
                    raise Exception("Primary model returned None")
            except Exception as e:
                # Try fallback model
                try:
                    result = self.robust_completion(prompt, fallback_model)
                    results.append({
                        "index": i,
                        "prompt": prompt,
                        "response": result,
                        "model_used": fallback_model,
                        "error": f"Fallback used: {e}"
                    })
                except Exception as fallback_error:
                    results.append({
                        "index": i,
                        "prompt": prompt,
                        "response": None,
                        "model_used": None,
                        "error": f"Both models failed: {e}, {fallback_error}"
                    })

        return results

# Example usage
robust_client = RobustInfernoClient()

# Single completion with retry
result = robust_client.robust_completion("Explain quantum computing")
if result:
    print(result["choices"][0]["text"])

# Completion with model fallback
result = robust_client.completion_with_fallback(
    "What is machine learning?",
    models=["gpt2", "distilgpt2"]
)

# Batch processing with fallback
prompts = [
    "Explain AI",
    "What is ML?",
    "Define neural networks"
]

batch_results = robust_client.batch_process_with_fallback(prompts)
for result in batch_results:
    if result["error"]:
        print(f"Error: {result['error']}")
    else:
        print(f"Success with {result['model_used']}: {result['response']['choices'][0]['text'][:50]}...")
```

## JavaScript/Node.js Examples

### Basic Node.js Client

```javascript
const axios = require('axios');
const EventSource = require('eventsource');

class InfernoClient {
    constructor(baseUrl = 'http://localhost:8080', apiKey = 'not-needed') {
        this.baseUrl = baseUrl.replace(/\/$/, '');
        this.apiKey = apiKey;

        this.client = axios.create({
            baseURL: `${this.baseUrl}/v1`,
            headers: {
                'Content-Type': 'application/json',
                ...(apiKey !== 'not-needed' && { 'Authorization': `Bearer ${apiKey}` })
            },
            timeout: 30000
        });
    }

    async healthCheck() {
        try {
            const response = await axios.get(`${this.baseUrl}/health`);
            return response.data;
        } catch (error) {
            throw new Error(`Health check failed: ${error.message}`);
        }
    }

    async listModels() {
        try {
            const response = await this.client.get('/models');
            return response.data.data.map(model => model.id);
        } catch (error) {
            throw new Error(`Failed to list models: ${error.message}`);
        }
    }

    async completion(prompt, options = {}) {
        const {
            model = 'gpt2',
            maxTokens = 100,
            temperature = 0.7,
            ...otherOptions
        } = options;

        try {
            const response = await this.client.post('/completions', {
                model,
                prompt,
                max_tokens: maxTokens,
                temperature,
                ...otherOptions
            });
            return response.data;
        } catch (error) {
            throw new Error(`Completion failed: ${error.message}`);
        }
    }

    async chatCompletion(messages, options = {}) {
        const {
            model = 'microsoft/DialoGPT-medium',
            maxTokens = 150,
            temperature = 0.7,
            ...otherOptions
        } = options;

        try {
            const response = await this.client.post('/chat/completions', {
                model,
                messages,
                max_tokens: maxTokens,
                temperature,
                ...otherOptions
            });
            return response.data;
        } catch (error) {
            throw new Error(`Chat completion failed: ${error.message}`);
        }
    }

    async embeddings(input, model = 'text-embedding-ada-002') {
        try {
            const response = await this.client.post('/embeddings', {
                model,
                input: Array.isArray(input) ? input : [input]
            });
            return response.data;
        } catch (error) {
            throw new Error(`Embeddings failed: ${error.message}`);
        }
    }

    streamCompletion(prompt, options = {}, onChunk, onError, onComplete) {
        const {
            model = 'gpt2',
            maxTokens = 200,
            temperature = 0.7,
            ...otherOptions
        } = options;

        const requestData = {
            model,
            prompt,
            max_tokens: maxTokens,
            temperature,
            stream: true,
            ...otherOptions
        };

        // Use fetch for streaming (Node.js 18+) or implement with EventSource
        this.client.post('/completions', requestData, {
            responseType: 'stream'
        }).then(response => {
            let buffer = '';

            response.data.on('data', (chunk) => {
                buffer += chunk.toString();
                const lines = buffer.split('\n');
                buffer = lines.pop(); // Keep incomplete line in buffer

                for (const line of lines) {
                    if (line.startsWith('data: ')) {
                        const data = line.slice(6);
                        if (data.trim() === '[DONE]') {
                            onComplete && onComplete();
                            return;
                        }

                        try {
                            const parsed = JSON.parse(data);
                            if (parsed.choices && parsed.choices[0].text) {
                                onChunk && onChunk(parsed.choices[0].text);
                            }
                        } catch (e) {
                            // Skip invalid JSON
                        }
                    }
                }
            });

            response.data.on('error', (error) => {
                onError && onError(error);
            });

        }).catch(error => {
            onError && onError(error);
        });
    }
}

// Example usage
async function main() {
    const client = new InfernoClient();

    try {
        // Health check
        const health = await client.healthCheck();
        console.log('Server status:', health.status);

        // List models
        const models = await client.listModels();
        console.log('Available models:', models);

        // Text completion
        const completion = await client.completion('The future of AI is', {
            maxTokens: 50
        });
        console.log('Completion:', completion.choices[0].text);

        // Chat completion
        const chat = await client.chatCompletion([
            { role: 'user', content: 'What is machine learning?' }
        ]);
        console.log('Chat response:', chat.choices[0].message.content);

        // Embeddings
        const embeddings = await client.embeddings(['Hello world', 'How are you?']);
        console.log('Embeddings shape:', embeddings.data.length, 'x', embeddings.data[0].embedding.length);

        // Streaming example
        console.log('Streaming response:');
        client.streamCompletion(
            'Write a short story about robots:',
            { maxTokens: 100 },
            (chunk) => process.stdout.write(chunk), // onChunk
            (error) => console.error('Stream error:', error), // onError
            () => console.log('\nStream completed') // onComplete
        );

    } catch (error) {
        console.error('Error:', error.message);
    }
}

// main();
```

### Browser JavaScript Example

```html
<!DOCTYPE html>
<html>
<head>
    <title>Inferno API Example</title>
</head>
<body>
    <div id="app">
        <h1>Inferno AI Chat</h1>

        <div id="chat-container">
            <div id="messages"></div>
            <input type="text" id="user-input" placeholder="Type your message...">
            <button onclick="sendMessage()">Send</button>
        </div>

        <div id="controls">
            <label>Model:</label>
            <select id="model-select">
                <option value="gpt2">GPT-2</option>
                <option value="microsoft/DialoGPT-medium">DialoGPT-Medium</option>
            </select>

            <label>Temperature:</label>
            <input type="range" id="temperature" min="0" max="2" step="0.1" value="0.7">
            <span id="temp-value">0.7</span>
        </div>
    </div>

    <script>
        class InfernoWebClient {
            constructor(baseUrl = 'http://localhost:8080') {
                this.baseUrl = baseUrl;
                this.conversationHistory = [];
            }

            async makeRequest(endpoint, options = {}) {
                const url = `${this.baseUrl}/v1${endpoint}`;

                const defaultOptions = {
                    method: 'GET',
                    headers: {
                        'Content-Type': 'application/json'
                    }
                };

                const finalOptions = { ...defaultOptions, ...options };

                try {
                    const response = await fetch(url, finalOptions);

                    if (!response.ok) {
                        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
                    }

                    return await response.json();
                } catch (error) {
                    console.error('Request failed:', error);
                    throw error;
                }
            }

            async chatCompletion(message, model = 'microsoft/DialoGPT-medium', temperature = 0.7) {
                const messages = [
                    ...this.conversationHistory,
                    { role: 'user', content: message }
                ];

                const data = await this.makeRequest('/chat/completions', {
                    method: 'POST',
                    body: JSON.stringify({
                        model,
                        messages,
                        max_tokens: 150,
                        temperature
                    })
                });

                const aiResponse = data.choices[0].message.content;

                // Update conversation history
                this.conversationHistory.push(
                    { role: 'user', content: message },
                    { role: 'assistant', content: aiResponse }
                );

                // Limit history to last 10 messages
                if (this.conversationHistory.length > 20) {
                    this.conversationHistory = this.conversationHistory.slice(-20);
                }

                return aiResponse;
            }

            async streamChatCompletion(message, model = 'microsoft/DialoGPT-medium', onChunk, onComplete) {
                const messages = [
                    ...this.conversationHistory,
                    { role: 'user', content: message }
                ];

                const response = await fetch(`${this.baseUrl}/v1/chat/completions`, {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json'
                    },
                    body: JSON.stringify({
                        model,
                        messages,
                        max_tokens: 150,
                        temperature: 0.7,
                        stream: true
                    })
                });

                const reader = response.body.getReader();
                const decoder = new TextDecoder();
                let fullResponse = '';

                try {
                    while (true) {
                        const { value, done } = await reader.read();
                        if (done) break;

                        const chunk = decoder.decode(value);
                        const lines = chunk.split('\n');

                        for (const line of lines) {
                            if (line.startsWith('data: ')) {
                                const data = line.slice(6);
                                if (data.trim() === '[DONE]') {
                                    onComplete && onComplete(fullResponse);
                                    return fullResponse;
                                }

                                try {
                                    const parsed = JSON.parse(data);
                                    const content = parsed.choices[0]?.delta?.content;
                                    if (content) {
                                        fullResponse += content;
                                        onChunk && onChunk(content);
                                    }
                                } catch (e) {
                                    // Skip invalid JSON
                                }
                            }
                        }
                    }
                } finally {
                    reader.releaseLock();
                }

                return fullResponse;
            }

            clearHistory() {
                this.conversationHistory = [];
            }
        }

        // Initialize client
        const client = new InfernoWebClient();

        // UI functions
        function addMessage(content, isUser = false) {
            const messagesDiv = document.getElementById('messages');
            const messageDiv = document.createElement('div');
            messageDiv.className = isUser ? 'user-message' : 'ai-message';
            messageDiv.style.cssText = `
                margin: 10px 0;
                padding: 10px;
                border-radius: 8px;
                ${isUser ? 'background-color: #e3f2fd; text-align: right;' : 'background-color: #f5f5f5;'}
            `;
            messageDiv.textContent = content;
            messagesDiv.appendChild(messageDiv);
            messagesDiv.scrollTop = messagesDiv.scrollHeight;
            return messageDiv;
        }

        async function sendMessage() {
            const input = document.getElementById('user-input');
            const message = input.value.trim();

            if (!message) return;

            const model = document.getElementById('model-select').value;
            const temperature = parseFloat(document.getElementById('temperature').value);

            // Add user message to UI
            addMessage(message, true);
            input.value = '';

            // Add placeholder for AI response
            const aiMessageDiv = addMessage('', false);

            try {
                // Stream the AI response
                await client.streamChatCompletion(
                    message,
                    model,
                    (chunk) => {
                        aiMessageDiv.textContent += chunk;
                    },
                    (fullResponse) => {
                        console.log('Response completed:', fullResponse);
                    }
                );
            } catch (error) {
                aiMessageDiv.textContent = `Error: ${error.message}`;
                aiMessageDiv.style.color = 'red';
            }
        }

        // Event listeners
        document.getElementById('user-input').addEventListener('keypress', function(e) {
            if (e.key === 'Enter') {
                sendMessage();
            }
        });

        document.getElementById('temperature').addEventListener('input', function(e) {
            document.getElementById('temp-value').textContent = e.target.value;
        });

        // Initialize UI
        document.getElementById('messages').style.cssText = `
            height: 400px;
            overflow-y: auto;
            border: 1px solid #ccc;
            padding: 10px;
            margin: 10px 0;
        `;
    </script>
</body>
</html>
```

## Go Examples

```go
package main

import (
    "bytes"
    "encoding/json"
    "fmt"
    "io"
    "net/http"
    "time"
)

type InfernoClient struct {
    BaseURL string
    APIKey  string
    Client  *http.Client
}

type CompletionRequest struct {
    Model       string  `json:"model"`
    Prompt      string  `json:"prompt"`
    MaxTokens   int     `json:"max_tokens"`
    Temperature float64 `json:"temperature"`
    Stream      bool    `json:"stream,omitempty"`
}

type CompletionResponse struct {
    ID      string `json:"id"`
    Object  string `json:"object"`
    Created int64  `json:"created"`
    Model   string `json:"model"`
    Choices []struct {
        Index        int    `json:"index"`
        Text         string `json:"text"`
        FinishReason string `json:"finish_reason"`
    } `json:"choices"`
    Usage struct {
        PromptTokens     int `json:"prompt_tokens"`
        CompletionTokens int `json:"completion_tokens"`
        TotalTokens      int `json:"total_tokens"`
    } `json:"usage"`
}

type ChatMessage struct {
    Role    string `json:"role"`
    Content string `json:"content"`
}

type ChatCompletionRequest struct {
    Model       string        `json:"model"`
    Messages    []ChatMessage `json:"messages"`
    MaxTokens   int           `json:"max_tokens"`
    Temperature float64       `json:"temperature"`
}

type ChatCompletionResponse struct {
    ID      string `json:"id"`
    Object  string `json:"object"`
    Created int64  `json:"created"`
    Model   string `json:"model"`
    Choices []struct {
        Index   int `json:"index"`
        Message struct {
            Role    string `json:"role"`
            Content string `json:"content"`
        } `json:"message"`
        FinishReason string `json:"finish_reason"`
    } `json:"choices"`
    Usage struct {
        PromptTokens     int `json:"prompt_tokens"`
        CompletionTokens int `json:"completion_tokens"`
        TotalTokens      int `json:"total_tokens"`
    } `json:"usage"`
}

func NewInfernoClient(baseURL, apiKey string) *InfernoClient {
    return &InfernoClient{
        BaseURL: baseURL,
        APIKey:  apiKey,
        Client: &http.Client{
            Timeout: 30 * time.Second,
        },
    }
}

func (c *InfernoClient) makeRequest(method, endpoint string, body interface{}) (*http.Response, error) {
    var reqBody io.Reader

    if body != nil {
        jsonData, err := json.Marshal(body)
        if err != nil {
            return nil, fmt.Errorf("failed to marshal request body: %w", err)
        }
        reqBody = bytes.NewBuffer(jsonData)
    }

    req, err := http.NewRequest(method, c.BaseURL+"/v1"+endpoint, reqBody)
    if err != nil {
        return nil, fmt.Errorf("failed to create request: %w", err)
    }

    req.Header.Set("Content-Type", "application/json")
    if c.APIKey != "not-needed" {
        req.Header.Set("Authorization", "Bearer "+c.APIKey)
    }

    resp, err := c.Client.Do(req)
    if err != nil {
        return nil, fmt.Errorf("failed to make request: %w", err)
    }

    if resp.StatusCode >= 400 {
        body, _ := io.ReadAll(resp.Body)
        resp.Body.Close()
        return nil, fmt.Errorf("HTTP %d: %s", resp.StatusCode, string(body))
    }

    return resp, nil
}

func (c *InfernoClient) Completion(prompt, model string, maxTokens int, temperature float64) (*CompletionResponse, error) {
    req := CompletionRequest{
        Model:       model,
        Prompt:      prompt,
        MaxTokens:   maxTokens,
        Temperature: temperature,
    }

    resp, err := c.makeRequest("POST", "/completions", req)
    if err != nil {
        return nil, err
    }
    defer resp.Body.Close()

    var result CompletionResponse
    if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
        return nil, fmt.Errorf("failed to decode response: %w", err)
    }

    return &result, nil
}

func (c *InfernoClient) ChatCompletion(messages []ChatMessage, model string, maxTokens int, temperature float64) (*ChatCompletionResponse, error) {
    req := ChatCompletionRequest{
        Model:       model,
        Messages:    messages,
        MaxTokens:   maxTokens,
        Temperature: temperature,
    }

    resp, err := c.makeRequest("POST", "/chat/completions", req)
    if err != nil {
        return nil, err
    }
    defer resp.Body.Close()

    var result ChatCompletionResponse
    if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
        return nil, fmt.Errorf("failed to decode response: %w", err)
    }

    return &result, nil
}

func (c *InfernoClient) ListModels() ([]string, error) {
    resp, err := c.makeRequest("GET", "/models", nil)
    if err != nil {
        return nil, err
    }
    defer resp.Body.Close()

    var result struct {
        Data []struct {
            ID string `json:"id"`
        } `json:"data"`
    }

    if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
        return nil, fmt.Errorf("failed to decode response: %w", err)
    }

    models := make([]string, len(result.Data))
    for i, model := range result.Data {
        models[i] = model.ID
    }

    return models, nil
}

func main() {
    // Initialize client
    client := NewInfernoClient("http://localhost:8080", "not-needed")

    // List models
    models, err := client.ListModels()
    if err != nil {
        fmt.Printf("Failed to list models: %v\n", err)
        return
    }
    fmt.Printf("Available models: %v\n", models)

    // Text completion
    completion, err := client.Completion(
        "The future of artificial intelligence is",
        "gpt2",
        100,
        0.7,
    )
    if err != nil {
        fmt.Printf("Completion failed: %v\n", err)
        return
    }
    fmt.Printf("Completion: %s\n", completion.Choices[0].Text)

    // Chat completion
    messages := []ChatMessage{
        {Role: "user", Content: "What is machine learning?"},
    }

    chat, err := client.ChatCompletion(
        messages,
        "microsoft/DialoGPT-medium",
        150,
        0.7,
    )
    if err != nil {
        fmt.Printf("Chat completion failed: %v\n", err)
        return
    }
    fmt.Printf("Chat response: %s\n", chat.Choices[0].Message.Content)
}
```

## Rust Examples

```rust
use reqwest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio;

#[derive(Debug, Serialize)]
struct CompletionRequest {
    model: String,
    prompt: String,
    max_tokens: u32,
    temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct CompletionResponse {
    id: String,
    object: String,
    created: i64,
    model: String,
    choices: Vec<Choice>,
    usage: Usage,
}

#[derive(Debug, Deserialize)]
struct Choice {
    index: u32,
    text: String,
    finish_reason: String,
}

#[derive(Debug, Deserialize)]
struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    id: String,
    object: String,
    created: i64,
    model: String,
    choices: Vec<ChatChoice>,
    usage: Usage,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    index: u32,
    message: ChatMessage,
    finish_reason: String,
}

struct InfernoClient {
    base_url: String,
    api_key: String,
    client: reqwest::Client,
}

impl InfernoClient {
    fn new(base_url: &str, api_key: &str) -> Self {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static("application/json"),
        );

        if api_key != "not-needed" {
            headers.insert(
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&format!("Bearer {}", api_key)).unwrap(),
            );
        }

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
            client,
        }
    }

    async fn completion(
        &self,
        prompt: &str,
        model: &str,
        max_tokens: u32,
        temperature: f32,
    ) -> Result<CompletionResponse, Box<dyn std::error::Error>> {
        let request = CompletionRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            max_tokens,
            temperature,
            stream: None,
        };

        let response = self
            .client
            .post(&format!("{}/v1/completions", self.base_url))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("HTTP {}: {}", response.status(), error_text).into());
        }

        let result: CompletionResponse = response.json().await?;
        Ok(result)
    }

    async fn chat_completion(
        &self,
        messages: Vec<ChatMessage>,
        model: &str,
        max_tokens: u32,
        temperature: f32,
    ) -> Result<ChatCompletionResponse, Box<dyn std::error::Error>> {
        let request = ChatCompletionRequest {
            model: model.to_string(),
            messages,
            max_tokens,
            temperature,
        };

        let response = self
            .client
            .post(&format!("{}/v1/chat/completions", self.base_url))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("HTTP {}: {}", response.status(), error_text).into());
        }

        let result: ChatCompletionResponse = response.json().await?;
        Ok(result)
    }

    async fn list_models(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let response = self
            .client
            .get(&format!("{}/v1/models", self.base_url))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("HTTP {}: {}", response.status(), error_text).into());
        }

        let result: serde_json::Value = response.json().await?;
        let models: Vec<String> = result["data"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|model| model["id"].as_str().map(String::from))
            .collect();

        Ok(models)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize client
    let client = InfernoClient::new("http://localhost:8080", "not-needed");

    // List models
    println!("Listing available models...");
    let models = client.list_models().await?;
    println!("Available models: {:?}", models);

    // Text completion
    println!("\nGenerating text completion...");
    let completion = client
        .completion(
            "The future of artificial intelligence is",
            "gpt2",
            100,
            0.7,
        )
        .await?;

    println!("Completion: {}", completion.choices[0].text);

    // Chat completion
    println!("\nGenerating chat completion...");
    let messages = vec![ChatMessage {
        role: "user".to_string(),
        content: "What is machine learning?".to_string(),
    }];

    let chat = client
        .chat_completion(messages, "microsoft/DialoGPT-medium", 150, 0.7)
        .await?;

    println!("Chat response: {}", chat.choices[0].message.content);

    Ok(())
}
```

This comprehensive REST API guide provides examples in multiple languages and covers all major use cases for integrating with Inferno. The examples progress from basic usage to production-ready patterns with error handling, retry logic, and performance optimizations.

## Next Steps

1. **[WebSocket Integration](websocket.md)** - Real-time streaming and bidirectional communication
2. **[Python Integration](python.md)** - Deep dive into Python-specific patterns
3. **[Performance Optimization](../tutorials/performance-optimization.md)** - Optimize your API integration