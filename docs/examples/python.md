# 🐍 Python Integration Guide

Complete guide for integrating Inferno with Python applications, from simple scripts to production web services.

## Overview

This guide covers:
- ✅ **OpenAI-compatible client** usage
- ✅ **Native Python SDK** integration
- ✅ **Async/await** patterns for high performance
- ✅ **Web framework** integration (FastAPI, Flask, Django)
- ✅ **Streaming responses** and real-time features
- ✅ **Error handling** and retry strategies
- ✅ **Production patterns** and best practices

## Quick Start

### Installation

```bash
# Install OpenAI Python client (recommended)
pip install openai>=1.0.0

# Or install with async support
pip install "openai[async]"

# Additional dependencies for examples
pip install requests httpx asyncio aiohttp fastapi uvicorn
```

### Basic Setup

```python
from openai import OpenAI

# Configure client for your Inferno instance
client = OpenAI(
    base_url="http://localhost:8080/v1",
    api_key="your-api-key"  # Use "not-needed" if auth is disabled
)

# Test connection
try:
    models = client.models.list()
    print(f"Available models: {[model.id for model in models.data]}")
except Exception as e:
    print(f"Connection failed: {e}")
```

## Basic Operations

### Simple Text Generation

```python
from openai import OpenAI

client = OpenAI(
    base_url="http://localhost:8080/v1",
    api_key="not-needed"
)

def generate_text(prompt, model="gpt2", max_tokens=100):
    """Generate text using Inferno."""
    try:
        response = client.completions.create(
            model=model,
            prompt=prompt,
            max_tokens=max_tokens,
            temperature=0.7
        )
        return response.choices[0].text.strip()
    except Exception as e:
        print(f"Error: {e}")
        return None

# Example usage
result = generate_text("The future of AI is", max_tokens=50)
print(f"Generated: {result}")
```

### Chat Completions

```python
def chat_with_ai(message, model="microsoft/DialoGPT-medium", history=None):
    """Chat with AI model maintaining conversation history."""
    if history is None:
        history = []

    # Add user message to history
    history.append({"role": "user", "content": message})

    try:
        response = client.chat.completions.create(
            model=model,
            messages=history,
            max_tokens=150,
            temperature=0.8
        )

        # Add AI response to history
        ai_message = response.choices[0].message.content
        history.append({"role": "assistant", "content": ai_message})

        return ai_message, history
    except Exception as e:
        print(f"Chat error: {e}")
        return None, history

# Example conversation
history = []
response, history = chat_with_ai("Hello! How are you?", history=history)
print(f"AI: {response}")

response, history = chat_with_ai("Tell me about machine learning", history=history)
print(f"AI: {response}")
```

### Text Embeddings

```python
import numpy as np
from sklearn.metrics.pairwise import cosine_similarity

def get_embeddings(texts, model="text-embedding-ada-002"):
    """Get embeddings for a list of texts."""
    try:
        response = client.embeddings.create(
            model=model,
            input=texts
        )
        return [data.embedding for data in response.data]
    except Exception as e:
        print(f"Embedding error: {e}")
        return None

def find_similar_texts(query, documents, model="text-embedding-ada-002"):
    """Find most similar documents to a query using embeddings."""
    # Get embeddings
    all_texts = [query] + documents
    embeddings = get_embeddings(all_texts, model)

    if not embeddings:
        return []

    query_embedding = np.array(embeddings[0]).reshape(1, -1)
    doc_embeddings = np.array(embeddings[1:])

    # Calculate similarities
    similarities = cosine_similarity(query_embedding, doc_embeddings)[0]

    # Rank documents by similarity
    ranked_docs = sorted(
        enumerate(documents),
        key=lambda x: similarities[x[0]],
        reverse=True
    )

    return [(doc, similarities[idx]) for idx, doc in ranked_docs]

# Example usage
documents = [
    "Machine learning is a subset of artificial intelligence",
    "Deep learning uses neural networks with multiple layers",
    "Python is a popular programming language",
    "Natural language processing helps computers understand text"
]

query = "What is AI?"
similar_docs = find_similar_texts(query, documents)

for doc, similarity in similar_docs:
    print(f"Similarity: {similarity:.3f} - {doc}")
```

## Streaming Responses

### Basic Streaming

```python
def stream_text(prompt, model="gpt2"):
    """Stream text generation in real-time."""
    try:
        stream = client.completions.create(
            model=model,
            prompt=prompt,
            max_tokens=200,
            stream=True,
            temperature=0.8
        )

        full_text = ""
        for chunk in stream:
            if chunk.choices[0].text:
                text_chunk = chunk.choices[0].text
                print(text_chunk, end="", flush=True)
                full_text += text_chunk

        print()  # New line after streaming
        return full_text

    except Exception as e:
        print(f"Streaming error: {e}")
        return ""

# Example usage
print("Streaming response:")
result = stream_text("Write a short story about a robot:")
```

### Streaming Chat

```python
def stream_chat(message, model="microsoft/DialoGPT-medium", history=None):
    """Stream chat responses in real-time."""
    if history is None:
        history = [{"role": "system", "content": "You are a helpful assistant."}]

    history.append({"role": "user", "content": message})

    try:
        stream = client.chat.completions.create(
            model=model,
            messages=history,
            max_tokens=200,
            stream=True,
            temperature=0.7
        )

        full_response = ""
        for chunk in stream:
            if chunk.choices[0].delta.content:
                content = chunk.choices[0].delta.content
                print(content, end="", flush=True)
                full_response += content

        print()  # New line
        history.append({"role": "assistant", "content": full_response})
        return full_response, history

    except Exception as e:
        print(f"Stream chat error: {e}")
        return "", history

# Interactive chat example
def interactive_chat():
    """Run an interactive chat session."""
    history = []
    print("Chat started! Type 'quit' to exit.")

    while True:
        user_input = input("\nYou: ")
        if user_input.lower() == 'quit':
            break

        print("AI: ", end="")
        response, history = stream_chat(user_input, history=history)

# Uncomment to run interactive chat
# interactive_chat()
```

## Async/Await Patterns

### Async Client Setup

```python
import asyncio
from openai import AsyncOpenAI

# Async client for high-performance applications
async_client = AsyncOpenAI(
    base_url="http://localhost:8080/v1",
    api_key="not-needed"
)

async def async_generate_text(prompt, model="gpt2"):
    """Async text generation."""
    try:
        response = await async_client.completions.create(
            model=model,
            prompt=prompt,
            max_tokens=100
        )
        return response.choices[0].text.strip()
    except Exception as e:
        print(f"Async error: {e}")
        return None

async def async_chat(message, model="microsoft/DialoGPT-medium"):
    """Async chat completion."""
    try:
        response = await async_client.chat.completions.create(
            model=model,
            messages=[{"role": "user", "content": message}],
            max_tokens=150
        )
        return response.choices[0].message.content
    except Exception as e:
        print(f"Async chat error: {e}")
        return None
```

### Concurrent Processing

```python
async def process_multiple_prompts(prompts, model="gpt2"):
    """Process multiple prompts concurrently."""
    tasks = [async_generate_text(prompt, model) for prompt in prompts]
    results = await asyncio.gather(*tasks, return_exceptions=True)

    # Filter out exceptions
    successful_results = [r for r in results if not isinstance(r, Exception)]
    failed_count = len(results) - len(successful_results)

    print(f"Processed {len(successful_results)} successfully, {failed_count} failed")
    return successful_results

async def batch_chat_processing(messages, model="microsoft/DialoGPT-medium"):
    """Process multiple chat messages concurrently."""
    tasks = [async_chat(msg, model) for msg in messages]
    responses = await asyncio.gather(*tasks, return_exceptions=True)

    results = []
    for i, (msg, resp) in enumerate(zip(messages, responses)):
        if isinstance(resp, Exception):
            results.append({"message": msg, "response": None, "error": str(resp)})
        else:
            results.append({"message": msg, "response": resp, "error": None})

    return results

# Example usage
async def main():
    prompts = [
        "Explain quantum computing",
        "What is machine learning?",
        "How does blockchain work?",
        "Describe neural networks"
    ]

    results = await process_multiple_prompts(prompts)
    for i, result in enumerate(results):
        print(f"Prompt {i+1}: {result[:100]}...")

# Run async example
# asyncio.run(main())
```

### Async Streaming

```python
async def async_stream_text(prompt, model="gpt2"):
    """Async streaming text generation."""
    try:
        stream = await async_client.completions.create(
            model=model,
            prompt=prompt,
            max_tokens=200,
            stream=True
        )

        full_text = ""
        async for chunk in stream:
            if chunk.choices[0].text:
                text_chunk = chunk.choices[0].text
                print(text_chunk, end="", flush=True)
                full_text += text_chunk

        print()
        return full_text

    except Exception as e:
        print(f"Async streaming error: {e}")
        return ""

async def async_stream_chat(message, model="microsoft/DialoGPT-medium"):
    """Async streaming chat."""
    try:
        stream = await async_client.chat.completions.create(
            model=model,
            messages=[{"role": "user", "content": message}],
            max_tokens=200,
            stream=True
        )

        full_response = ""
        async for chunk in stream:
            if chunk.choices[0].delta.content:
                content = chunk.choices[0].delta.content
                print(content, end="", flush=True)
                full_response += content

        print()
        return full_response

    except Exception as e:
        print(f"Async stream chat error: {e}")
        return ""
```

## Web Framework Integration

### FastAPI Integration

```python
from fastapi import FastAPI, HTTPException, BackgroundTasks
from fastapi.responses import StreamingResponse
from pydantic import BaseModel
import json
import asyncio

app = FastAPI(title="Inferno AI API", version="1.0.0")

# Request/Response models
class CompletionRequest(BaseModel):
    prompt: str
    model: str = "gpt2"
    max_tokens: int = 100
    temperature: float = 0.7
    stream: bool = False

class ChatRequest(BaseModel):
    message: str
    model: str = "microsoft/DialoGPT-medium"
    max_tokens: int = 150
    temperature: float = 0.7
    stream: bool = False
    history: list = []

class CompletionResponse(BaseModel):
    text: str
    model: str
    usage: dict

@app.on_event("startup")
async def startup_event():
    """Initialize connections on startup."""
    try:
        # Test connection to Inferno
        models = await async_client.models.list()
        print(f"Connected to Inferno. Available models: {len(models.data)}")
    except Exception as e:
        print(f"Warning: Could not connect to Inferno: {e}")

@app.get("/")
async def root():
    """Health check endpoint."""
    return {"status": "healthy", "service": "Inferno AI API"}

@app.post("/completion", response_model=CompletionResponse)
async def create_completion(request: CompletionRequest):
    """Generate text completion."""
    try:
        if request.stream:
            raise HTTPException(status_code=400, detail="Use /completion/stream for streaming")

        response = await async_client.completions.create(
            model=request.model,
            prompt=request.prompt,
            max_tokens=request.max_tokens,
            temperature=request.temperature
        )

        return CompletionResponse(
            text=response.choices[0].text.strip(),
            model=request.model,
            usage=response.usage.dict()
        )
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@app.post("/completion/stream")
async def stream_completion(request: CompletionRequest):
    """Stream text completion."""
    try:
        async def generate():
            stream = await async_client.completions.create(
                model=request.model,
                prompt=request.prompt,
                max_tokens=request.max_tokens,
                temperature=request.temperature,
                stream=True
            )

            async for chunk in stream:
                if chunk.choices[0].text:
                    yield f"data: {json.dumps({'text': chunk.choices[0].text})}\\n\\n"

            yield "data: [DONE]\\n\\n"

        return StreamingResponse(
            generate(),
            media_type="text/plain",
            headers={"Cache-Control": "no-cache"}
        )
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@app.post("/chat")
async def chat_completion(request: ChatRequest):
    """Chat completion."""
    try:
        messages = request.history + [{"role": "user", "content": request.message}]

        if request.stream:
            raise HTTPException(status_code=400, detail="Use /chat/stream for streaming")

        response = await async_client.chat.completions.create(
            model=request.model,
            messages=messages,
            max_tokens=request.max_tokens,
            temperature=request.temperature
        )

        ai_response = response.choices[0].message.content
        updated_history = messages + [{"role": "assistant", "content": ai_response}]

        return {
            "response": ai_response,
            "history": updated_history,
            "usage": response.usage.dict()
        }
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@app.post("/chat/stream")
async def stream_chat(request: ChatRequest):
    """Stream chat completion."""
    try:
        messages = request.history + [{"role": "user", "content": request.message}]

        async def generate():
            stream = await async_client.chat.completions.create(
                model=request.model,
                messages=messages,
                max_tokens=request.max_tokens,
                temperature=request.temperature,
                stream=True
            )

            async for chunk in stream:
                if chunk.choices[0].delta.content:
                    yield f"data: {json.dumps({'content': chunk.choices[0].delta.content})}\\n\\n"

            yield "data: [DONE]\\n\\n"

        return StreamingResponse(
            generate(),
            media_type="text/plain",
            headers={"Cache-Control": "no-cache"}
        )
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/models")
async def list_models():
    """List available models."""
    try:
        models = await async_client.models.list()
        return {"models": [model.id for model in models.data]}
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

# Run with: uvicorn main:app --reload --port 8000
```

### Flask Integration

```python
from flask import Flask, request, jsonify, Response, stream_template
import json

app = Flask(__name__)

@app.route('/')
def health_check():
    """Health check endpoint."""
    return jsonify({"status": "healthy", "service": "Inferno Flask API"})

@app.route('/completion', methods=['POST'])
def completion():
    """Text completion endpoint."""
    try:
        data = request.get_json()
        prompt = data.get('prompt')
        model = data.get('model', 'gpt2')
        max_tokens = data.get('max_tokens', 100)

        response = client.completions.create(
            model=model,
            prompt=prompt,
            max_tokens=max_tokens
        )

        return jsonify({
            'text': response.choices[0].text.strip(),
            'model': model,
            'usage': response.usage.dict()
        })
    except Exception as e:
        return jsonify({'error': str(e)}), 500

@app.route('/completion/stream', methods=['POST'])
def stream_completion():
    """Streaming text completion."""
    try:
        data = request.get_json()
        prompt = data.get('prompt')
        model = data.get('model', 'gpt2')
        max_tokens = data.get('max_tokens', 100)

        def generate():
            stream = client.completions.create(
                model=model,
                prompt=prompt,
                max_tokens=max_tokens,
                stream=True
            )

            for chunk in stream:
                if chunk.choices[0].text:
                    yield f"data: {json.dumps({'text': chunk.choices[0].text})}\\n\\n"
            yield "data: [DONE]\\n\\n"

        return Response(generate(), mimetype='text/plain')
    except Exception as e:
        return jsonify({'error': str(e)}), 500

@app.route('/chat', methods=['POST'])
def chat():
    """Chat completion endpoint."""
    try:
        data = request.get_json()
        message = data.get('message')
        model = data.get('model', 'microsoft/DialoGPT-medium')
        history = data.get('history', [])

        messages = history + [{"role": "user", "content": message}]

        response = client.chat.completions.create(
            model=model,
            messages=messages,
            max_tokens=150
        )

        ai_response = response.choices[0].message.content
        updated_history = messages + [{"role": "assistant", "content": ai_response}]

        return jsonify({
            'response': ai_response,
            'history': updated_history,
            'usage': response.usage.dict()
        })
    except Exception as e:
        return jsonify({'error': str(e)}), 500

if __name__ == '__main__':
    app.run(debug=True, port=5000)
```

### Django Integration

```python
# views.py
from django.http import JsonResponse, StreamingHttpResponse
from django.views.decorators.csrf import csrf_exempt
from django.views.decorators.http import require_http_methods
import json

@csrf_exempt
@require_http_methods(["POST"])
def completion_view(request):
    """Django view for text completion."""
    try:
        data = json.loads(request.body)
        prompt = data.get('prompt')
        model = data.get('model', 'gpt2')

        response = client.completions.create(
            model=model,
            prompt=prompt,
            max_tokens=100
        )

        return JsonResponse({
            'text': response.choices[0].text.strip(),
            'model': model,
            'usage': response.usage.dict()
        })
    except Exception as e:
        return JsonResponse({'error': str(e)}, status=500)

@csrf_exempt
@require_http_methods(["POST"])
def chat_view(request):
    """Django view for chat completion."""
    try:
        data = json.loads(request.body)
        message = data.get('message')
        model = data.get('model', 'microsoft/DialoGPT-medium')

        response = client.chat.completions.create(
            model=model,
            messages=[{"role": "user", "content": message}],
            max_tokens=150
        )

        return JsonResponse({
            'response': response.choices[0].message.content,
            'usage': response.usage.dict()
        })
    except Exception as e:
        return JsonResponse({'error': str(e)}, status=500)

# urls.py
from django.urls import path
from . import views

urlpatterns = [
    path('completion/', views.completion_view, name='completion'),
    path('chat/', views.chat_view, name='chat'),
]
```

## Error Handling and Retry Logic

### Robust Error Handling

```python
import time
import random
from functools import wraps
from openai import OpenAI, OpenAIError

def retry_with_exponential_backoff(
    max_retries=5,
    initial_delay=1,
    exponential_base=2,
    jitter=True,
    max_delay=60
):
    """Decorator for retrying functions with exponential backoff."""
    def decorator(func):
        @wraps(func)
        def wrapper(*args, **kwargs):
            delay = initial_delay

            for attempt in range(max_retries):
                try:
                    return func(*args, **kwargs)
                except OpenAIError as e:
                    if attempt == max_retries - 1:
                        raise e

                    # Add jitter to prevent thundering herd
                    if jitter:
                        delay_with_jitter = delay * (0.5 + random.random() * 0.5)
                    else:
                        delay_with_jitter = delay

                    print(f"Attempt {attempt + 1} failed: {e}. Retrying in {delay_with_jitter:.2f}s...")
                    time.sleep(min(delay_with_jitter, max_delay))
                    delay *= exponential_base

            return None
        return wrapper
    return decorator

class InfernoClient:
    """Enhanced Inferno client with error handling and retry logic."""

    def __init__(self, base_url="http://localhost:8080/v1", api_key="not-needed"):
        self.client = OpenAI(base_url=base_url, api_key=api_key)
        self.base_url = base_url

    def is_healthy(self):
        """Check if Inferno server is healthy."""
        try:
            import requests
            response = requests.get(f"{self.base_url.replace('/v1', '')}/health", timeout=5)
            return response.status_code == 200
        except:
            return False

    @retry_with_exponential_backoff(max_retries=3)
    def safe_completion(self, prompt, model="gpt2", **kwargs):
        """Safe text completion with retry logic."""
        if not self.is_healthy():
            raise ConnectionError("Inferno server is not healthy")

        return self.client.completions.create(
            model=model,
            prompt=prompt,
            **kwargs
        )

    @retry_with_exponential_backoff(max_retries=3)
    def safe_chat_completion(self, messages, model="microsoft/DialoGPT-medium", **kwargs):
        """Safe chat completion with retry logic."""
        if not self.is_healthy():
            raise ConnectionError("Inferno server is not healthy")

        return self.client.chat.completions.create(
            model=model,
            messages=messages,
            **kwargs
        )

    def batch_process_with_fallback(self, prompts, model="gpt2", fallback_model="distilgpt2"):
        """Batch process with fallback model on failures."""
        results = []

        for prompt in prompts:
            try:
                response = self.safe_completion(prompt, model=model)
                results.append({
                    "prompt": prompt,
                    "response": response.choices[0].text.strip(),
                    "model": model,
                    "error": None
                })
            except Exception as e:
                print(f"Primary model failed for prompt '{prompt[:50]}...': {e}")

                # Try fallback model
                try:
                    response = self.safe_completion(prompt, model=fallback_model)
                    results.append({
                        "prompt": prompt,
                        "response": response.choices[0].text.strip(),
                        "model": fallback_model,
                        "error": f"Fallback used: {e}"
                    })
                except Exception as fallback_error:
                    results.append({
                        "prompt": prompt,
                        "response": None,
                        "model": None,
                        "error": f"Both models failed: {e}, {fallback_error}"
                    })

        return results

# Example usage
inferno = InfernoClient()

# Test with error handling
try:
    response = inferno.safe_completion("Explain quantum computing", max_tokens=100)
    print(f"Success: {response.choices[0].text}")
except Exception as e:
    print(f"All retries failed: {e}")

# Batch processing with fallback
prompts = [
    "What is machine learning?",
    "Explain neural networks",
    "How does AI work?"
]

results = inferno.batch_process_with_fallback(prompts)
for result in results:
    if result["error"]:
        print(f"Error for '{result['prompt'][:30]}...': {result['error']}")
    else:
        print(f"Success: {result['response'][:50]}...")
```

## Advanced Usage Patterns

### Model Management

```python
class ModelManager:
    """Manage multiple models and route requests intelligently."""

    def __init__(self, base_url="http://localhost:8080/v1", api_key="not-needed"):
        self.client = OpenAI(base_url=base_url, api_key=api_key)
        self.model_capabilities = {
            "gpt2": {"type": "generation", "speed": "fast", "quality": "medium"},
            "microsoft/DialoGPT-medium": {"type": "chat", "speed": "medium", "quality": "high"},
            "bert-base": {"type": "embeddings", "speed": "fast", "quality": "high"},
            "codellama/CodeLlama-7b": {"type": "code", "speed": "slow", "quality": "high"}
        }

    def get_available_models(self):
        """Get list of available models."""
        try:
            models = self.client.models.list()
            return [model.id for model in models.data]
        except Exception as e:
            print(f"Error fetching models: {e}")
            return []

    def select_best_model(self, task_type, priority="balanced"):
        """Select the best model for a task."""
        available = self.get_available_models()
        suitable_models = []

        for model, caps in self.model_capabilities.items():
            if model in available and caps["type"] == task_type:
                suitable_models.append((model, caps))

        if not suitable_models:
            return None

        # Sort by priority
        if priority == "speed":
            suitable_models.sort(key=lambda x: x[1]["speed"] == "fast", reverse=True)
        elif priority == "quality":
            suitable_models.sort(key=lambda x: x[1]["quality"] == "high", reverse=True)

        return suitable_models[0][0]

    def smart_completion(self, prompt, task_type="generation", priority="balanced"):
        """Intelligent completion with automatic model selection."""
        model = self.select_best_model(task_type, priority)
        if not model:
            raise ValueError(f"No suitable model found for task: {task_type}")

        print(f"Using model: {model} for {task_type}")

        if task_type == "chat":
            return self.client.chat.completions.create(
                model=model,
                messages=[{"role": "user", "content": prompt}]
            )
        else:
            return self.client.completions.create(
                model=model,
                prompt=prompt
            )

# Example usage
manager = ModelManager()

# Smart model selection
response = manager.smart_completion("Hello, how are you?", task_type="chat", priority="quality")
print(response.choices[0].message.content)

response = manager.smart_completion("Quick summary:", task_type="generation", priority="speed")
print(response.choices[0].text)
```

### Caching and Performance

```python
import hashlib
import pickle
import time
from functools import wraps

class InfernoCache:
    """Simple in-memory cache for Inferno responses."""

    def __init__(self, max_size=1000, ttl=3600):
        self.cache = {}
        self.access_times = {}
        self.max_size = max_size
        self.ttl = ttl

    def _get_cache_key(self, prompt, model, **kwargs):
        """Generate cache key from request parameters."""
        key_data = f"{prompt}:{model}:{sorted(kwargs.items())}"
        return hashlib.md5(key_data.encode()).hexdigest()

    def _is_expired(self, timestamp):
        """Check if cache entry is expired."""
        return time.time() - timestamp > self.ttl

    def _evict_lru(self):
        """Evict least recently used items."""
        if len(self.cache) >= self.max_size:
            lru_key = min(self.access_times.keys(), key=self.access_times.get)
            del self.cache[lru_key]
            del self.access_times[lru_key]

    def get(self, prompt, model, **kwargs):
        """Get cached response."""
        key = self._get_cache_key(prompt, model, **kwargs)

        if key in self.cache:
            entry = self.cache[key]
            if not self._is_expired(entry['timestamp']):
                self.access_times[key] = time.time()
                return entry['response']
            else:
                # Remove expired entry
                del self.cache[key]
                del self.access_times[key]

        return None

    def set(self, prompt, model, response, **kwargs):
        """Cache response."""
        key = self._get_cache_key(prompt, model, **kwargs)

        self._evict_lru()

        self.cache[key] = {
            'response': response,
            'timestamp': time.time()
        }
        self.access_times[key] = time.time()

    def clear(self):
        """Clear all cache."""
        self.cache.clear()
        self.access_times.clear()

    def stats(self):
        """Get cache statistics."""
        total_entries = len(self.cache)
        expired_entries = sum(1 for entry in self.cache.values()
                             if self._is_expired(entry['timestamp']))

        return {
            'total_entries': total_entries,
            'active_entries': total_entries - expired_entries,
            'expired_entries': expired_entries,
            'cache_size': total_entries,
            'max_size': self.max_size
        }

def with_cache(cache_instance):
    """Decorator to add caching to Inferno functions."""
    def decorator(func):
        @wraps(func)
        def wrapper(*args, **kwargs):
            # Extract prompt and model for cache key
            prompt = args[0] if args else kwargs.get('prompt', '')
            model = kwargs.get('model', 'gpt2')

            # Try to get from cache
            cached_response = cache_instance.get(prompt, model, **kwargs)
            if cached_response:
                print(f"Cache hit for prompt: {prompt[:30]}...")
                return cached_response

            # Call original function
            print(f"Cache miss for prompt: {prompt[:30]}...")
            response = func(*args, **kwargs)

            # Cache the response
            cache_instance.set(prompt, model, response, **kwargs)

            return response
        return wrapper
    return decorator

# Create cache instance
cache = InfernoCache(max_size=500, ttl=1800)  # 30 minutes TTL

@with_cache(cache)
def cached_completion(prompt, model="gpt2", **kwargs):
    """Cached text completion."""
    return client.completions.create(
        model=model,
        prompt=prompt,
        **kwargs
    )

# Example usage
response1 = cached_completion("What is AI?", max_tokens=50)  # Cache miss
response2 = cached_completion("What is AI?", max_tokens=50)  # Cache hit

print(f"Cache stats: {cache.stats()}")
```

### Production Monitoring

```python
import logging
import time
import psutil
from datetime import datetime

class InfernoMonitor:
    """Monitor Inferno client performance and health."""

    def __init__(self, client, log_file="inferno_client.log"):
        self.client = client
        self.logger = self._setup_logger(log_file)
        self.metrics = {
            'requests_total': 0,
            'requests_success': 0,
            'requests_failed': 0,
            'total_latency': 0,
            'avg_latency': 0
        }

    def _setup_logger(self, log_file):
        """Set up logging configuration."""
        logger = logging.getLogger('inferno_client')
        logger.setLevel(logging.INFO)

        handler = logging.FileHandler(log_file)
        formatter = logging.Formatter(
            '%(asctime)s - %(name)s - %(levelname)s - %(message)s'
        )
        handler.setFormatter(formatter)
        logger.addHandler(handler)

        return logger

    def _update_metrics(self, success, latency):
        """Update performance metrics."""
        self.metrics['requests_total'] += 1
        if success:
            self.metrics['requests_success'] += 1
        else:
            self.metrics['requests_failed'] += 1

        self.metrics['total_latency'] += latency
        self.metrics['avg_latency'] = (
            self.metrics['total_latency'] / self.metrics['requests_total']
        )

    def monitored_completion(self, prompt, model="gpt2", **kwargs):
        """Completion with monitoring."""
        start_time = time.time()

        try:
            self.logger.info(f"Starting completion request - Model: {model}, Prompt length: {len(prompt)}")

            response = self.client.completions.create(
                model=model,
                prompt=prompt,
                **kwargs
            )

            latency = time.time() - start_time
            self._update_metrics(True, latency)

            self.logger.info(f"Completion successful - Latency: {latency:.2f}s, Tokens: {response.usage.total_tokens}")

            return response

        except Exception as e:
            latency = time.time() - start_time
            self._update_metrics(False, latency)

            self.logger.error(f"Completion failed - Latency: {latency:.2f}s, Error: {e}")
            raise

    def get_system_metrics(self):
        """Get system resource metrics."""
        return {
            'cpu_percent': psutil.cpu_percent(),
            'memory_percent': psutil.virtual_memory().percent,
            'disk_usage': psutil.disk_usage('/').percent,
            'timestamp': datetime.now().isoformat()
        }

    def get_performance_report(self):
        """Generate performance report."""
        success_rate = (
            (self.metrics['requests_success'] / self.metrics['requests_total'] * 100)
            if self.metrics['requests_total'] > 0 else 0
        )

        report = {
            'performance_metrics': self.metrics.copy(),
            'success_rate': f"{success_rate:.2f}%",
            'system_metrics': self.get_system_metrics(),
            'report_time': datetime.now().isoformat()
        }

        return report

    def log_report(self):
        """Log performance report."""
        report = self.get_performance_report()
        self.logger.info(f"Performance Report: {report}")
        return report

# Example usage
monitor = InfernoMonitor(client)

# Monitored requests
try:
    response = monitor.monitored_completion("Explain AI", model="gpt2", max_tokens=100)
    print(f"Response: {response.choices[0].text[:50]}...")
except Exception as e:
    print(f"Request failed: {e}")

# Get performance report
report = monitor.log_report()
print(f"Success rate: {report['success_rate']}")
print(f"Average latency: {report['performance_metrics']['avg_latency']:.2f}s")
```

## Testing and Development

### Unit Testing

```python
import unittest
from unittest.mock import Mock, patch
import json

class TestInfernoIntegration(unittest.TestCase):
    """Unit tests for Inferno integration."""

    def setUp(self):
        """Set up test fixtures."""
        self.client = OpenAI(
            base_url="http://localhost:8080/v1",
            api_key="test-key"
        )

    @patch('openai.OpenAI')
    def test_basic_completion(self, mock_openai):
        """Test basic text completion."""
        # Mock response
        mock_response = Mock()
        mock_response.choices = [Mock(text="This is a test response")]
        mock_response.usage = Mock(total_tokens=20)

        mock_client = Mock()
        mock_client.completions.create.return_value = mock_response
        mock_openai.return_value = mock_client

        # Test
        client = OpenAI(base_url="http://test", api_key="test")
        response = client.completions.create(
            model="test-model",
            prompt="test prompt"
        )

        self.assertEqual(response.choices[0].text, "This is a test response")
        self.assertEqual(response.usage.total_tokens, 20)

    @patch('requests.get')
    def test_health_check(self, mock_get):
        """Test health check functionality."""
        mock_get.return_value.status_code = 200
        mock_get.return_value.json.return_value = {"status": "healthy"}

        import requests
        response = requests.get("http://localhost:8080/health")

        self.assertEqual(response.status_code, 200)
        self.assertEqual(response.json()["status"], "healthy")

    def test_cache_functionality(self):
        """Test caching functionality."""
        cache = InfernoCache(max_size=10, ttl=60)

        # Test cache miss
        result = cache.get("test prompt", "test-model")
        self.assertIsNone(result)

        # Test cache set and hit
        mock_response = {"text": "cached response"}
        cache.set("test prompt", "test-model", mock_response)

        result = cache.get("test prompt", "test-model")
        self.assertEqual(result, mock_response)

        # Test cache stats
        stats = cache.stats()
        self.assertEqual(stats['total_entries'], 1)

if __name__ == '__main__':
    unittest.main()
```

### Integration Testing

```python
import asyncio
import pytest

@pytest.mark.asyncio
async def test_async_completion():
    """Test async completion functionality."""
    async_client = AsyncOpenAI(
        base_url="http://localhost:8080/v1",
        api_key="not-needed"
    )

    try:
        response = await async_client.completions.create(
            model="gpt2",
            prompt="Test prompt",
            max_tokens=10
        )

        assert response.choices[0].text is not None
        assert len(response.choices[0].text) > 0

    except Exception as e:
        pytest.skip(f"Inferno server not available: {e}")

def test_error_handling():
    """Test error handling with invalid requests."""
    client = OpenAI(
        base_url="http://localhost:8080/v1",
        api_key="not-needed"
    )

    # Test with invalid model
    with pytest.raises(Exception):
        client.completions.create(
            model="nonexistent-model",
            prompt="test"
        )

def test_streaming():
    """Test streaming functionality."""
    client = OpenAI(
        base_url="http://localhost:8080/v1",
        api_key="not-needed"
    )

    try:
        stream = client.completions.create(
            model="gpt2",
            prompt="Test streaming",
            max_tokens=20,
            stream=True
        )

        chunks = []
        for chunk in stream:
            if chunk.choices[0].text:
                chunks.append(chunk.choices[0].text)

        assert len(chunks) > 0

    except Exception as e:
        pytest.skip(f"Inferno server not available: {e}")

# Run tests with: pytest test_inferno.py -v
```

## Best Practices

### Configuration Management

```python
import os
from dataclasses import dataclass
from typing import Optional

@dataclass
class InfernoConfig:
    """Configuration for Inferno client."""
    base_url: str = "http://localhost:8080/v1"
    api_key: str = "not-needed"
    timeout: int = 30
    max_retries: int = 3
    default_model: str = "gpt2"
    max_tokens: int = 100
    temperature: float = 0.7
    enable_caching: bool = True
    cache_ttl: int = 3600
    log_level: str = "INFO"

def load_config() -> InfernoConfig:
    """Load configuration from environment variables."""
    return InfernoConfig(
        base_url=os.getenv("INFERNO_BASE_URL", "http://localhost:8080/v1"),
        api_key=os.getenv("INFERNO_API_KEY", "not-needed"),
        timeout=int(os.getenv("INFERNO_TIMEOUT", "30")),
        max_retries=int(os.getenv("INFERNO_MAX_RETRIES", "3")),
        default_model=os.getenv("INFERNO_DEFAULT_MODEL", "gpt2"),
        max_tokens=int(os.getenv("INFERNO_MAX_TOKENS", "100")),
        temperature=float(os.getenv("INFERNO_TEMPERATURE", "0.7")),
        enable_caching=os.getenv("INFERNO_ENABLE_CACHING", "true").lower() == "true",
        cache_ttl=int(os.getenv("INFERNO_CACHE_TTL", "3600")),
        log_level=os.getenv("INFERNO_LOG_LEVEL", "INFO")
    )

class InfernoClientFactory:
    """Factory for creating configured Inferno clients."""

    @staticmethod
    def create_client(config: Optional[InfernoConfig] = None) -> OpenAI:
        """Create configured Inferno client."""
        if config is None:
            config = load_config()

        return OpenAI(
            base_url=config.base_url,
            api_key=config.api_key,
            timeout=config.timeout
        )

    @staticmethod
    def create_async_client(config: Optional[InfernoConfig] = None) -> AsyncOpenAI:
        """Create configured async Inferno client."""
        if config is None:
            config = load_config()

        return AsyncOpenAI(
            base_url=config.base_url,
            api_key=config.api_key,
            timeout=config.timeout
        )

# Example usage
config = load_config()
client = InfernoClientFactory.create_client(config)
```

### Production Deployment

```python
# production_app.py
import logging
import sys
from contextlib import asynccontextmanager
from fastapi import FastAPI
from openai import AsyncOpenAI

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
    handlers=[
        logging.FileHandler('inferno_app.log'),
        logging.StreamHandler(sys.stdout)
    ]
)

logger = logging.getLogger(__name__)

# Global client instance
client = None

@asynccontextmanager
async def lifespan(app: FastAPI):
    """Manage application lifecycle."""
    global client

    # Startup
    logger.info("Starting Inferno application...")
    config = load_config()
    client = InfernoClientFactory.create_async_client(config)

    # Test connection
    try:
        models = await client.models.list()
        logger.info(f"Connected to Inferno. Available models: {len(models.data)}")
    except Exception as e:
        logger.error(f"Failed to connect to Inferno: {e}")
        sys.exit(1)

    yield

    # Shutdown
    logger.info("Shutting down Inferno application...")
    await client.aclose()

app = FastAPI(
    title="Production Inferno API",
    version="1.0.0",
    lifespan=lifespan
)

# Include your API routes here
# from .routes import completion, chat, embeddings
# app.include_router(completion.router)
# app.include_router(chat.router)
# app.include_router(embeddings.router)

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(
        "production_app:app",
        host="0.0.0.0",
        port=8000,
        workers=4,
        access_log=True
    )
```

This comprehensive Python integration guide provides everything you need to build production-ready applications with Inferno. From simple scripts to complex web services, these examples show how to leverage Inferno's power through Python's ecosystem.

## Next Steps

1. **[WebSocket Integration](websocket.md)** - Real-time streaming with WebSockets
2. **[REST API Examples](rest-api.md)** - Complete API usage examples
3. **[Performance Optimization](../tutorials/performance-optimization.md)** - Optimize your Python integration