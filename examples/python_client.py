#!/usr/bin/env python3
"""
Inferno Python Client Example

This example demonstrates how to use the Inferno API with Python.
Includes basic inference, streaming, batch processing, and more.
"""

import json
import time
import asyncio
import requests
import websocket
from typing import List, Dict, Optional, Generator
import sseclient


class InfernoClient:
    """Simple client for Inferno AI/ML inference server."""

    def __init__(self, base_url: str = "http://localhost:8080", api_key: Optional[str] = None):
        """
        Initialize the Inferno client.

        Args:
            base_url: The base URL of the Inferno server
            api_key: Optional API key for authentication
        """
        self.base_url = base_url.rstrip('/')
        self.api_key = api_key
        self.session = requests.Session()

        if api_key:
            self.session.headers['Authorization'] = f'Bearer {api_key}'
        self.session.headers['Content-Type'] = 'application/json'

    def health_check(self) -> Dict:
        """Check the health status of the server."""
        response = self.session.get(f'{self.base_url}/health')
        response.raise_for_status()
        return response.json()

    def list_models(self) -> List[Dict]:
        """List all available models."""
        response = self.session.get(f'{self.base_url}/models')
        response.raise_for_status()
        return response.json()['models']

    def load_model(self, model_id: str, **kwargs) -> Dict:
        """
        Load a model into memory.

        Args:
            model_id: The ID of the model to load
            **kwargs: Additional parameters (gpu_layers, context_size, etc.)
        """
        response = self.session.post(
            f'{self.base_url}/models/{model_id}/load',
            json=kwargs
        )
        response.raise_for_status()
        return response.json()

    def unload_model(self, model_id: str) -> Dict:
        """Unload a model from memory."""
        response = self.session.post(f'{self.base_url}/models/{model_id}/unload')
        response.raise_for_status()
        return response.json()

    def inference(self,
                 model: str,
                 prompt: str,
                 max_tokens: int = 100,
                 temperature: float = 0.7,
                 top_p: float = 0.9,
                 top_k: int = 40,
                 **kwargs) -> str:
        """
        Run synchronous inference.

        Args:
            model: Model ID to use
            prompt: Input prompt
            max_tokens: Maximum tokens to generate
            temperature: Sampling temperature
            top_p: Nucleus sampling parameter
            top_k: Top-k sampling parameter
            **kwargs: Additional parameters

        Returns:
            Generated text
        """
        request_data = {
            'model': model,
            'prompt': prompt,
            'max_tokens': max_tokens,
            'temperature': temperature,
            'top_p': top_p,
            'top_k': top_k,
            'stream': False,
            **kwargs
        }

        response = self.session.post(
            f'{self.base_url}/inference',
            json=request_data
        )
        response.raise_for_status()

        result = response.json()
        return result['choices'][0]['text']

    def stream_inference(self,
                        model: str,
                        prompt: str,
                        max_tokens: int = 100,
                        **kwargs) -> Generator[str, None, None]:
        """
        Stream inference results using Server-Sent Events.

        Args:
            model: Model ID to use
            prompt: Input prompt
            max_tokens: Maximum tokens to generate
            **kwargs: Additional parameters

        Yields:
            Generated tokens as they become available
        """
        request_data = {
            'model': model,
            'prompt': prompt,
            'max_tokens': max_tokens,
            'stream': True,
            **kwargs
        }

        response = self.session.post(
            f'{self.base_url}/inference/stream',
            json=request_data,
            stream=True,
            headers={'Accept': 'text/event-stream'}
        )
        response.raise_for_status()

        client = sseclient.SSEClient(response)
        for event in client.events():
            data = json.loads(event.data)
            if 'token' in data:
                yield data['token']
            elif 'done' in data:
                break
            elif 'error' in data:
                raise Exception(f"Stream error: {data['error']}")

    def embeddings(self, model: str, texts: List[str]) -> List[List[float]]:
        """
        Generate embeddings for text inputs.

        Args:
            model: Model ID to use
            texts: List of texts to embed

        Returns:
            List of embedding vectors
        """
        request_data = {
            'model': model,
            'input': texts,
            'encoding_format': 'float'
        }

        response = self.session.post(
            f'{self.base_url}/embeddings',
            json=request_data
        )
        response.raise_for_status()

        result = response.json()
        return [item['embedding'] for item in result['data']]

    def chat_completion(self,
                       model: str,
                       messages: List[Dict[str, str]],
                       **kwargs) -> str:
        """
        OpenAI-compatible chat completion.

        Args:
            model: Model ID to use
            messages: List of message dictionaries with 'role' and 'content'
            **kwargs: Additional parameters

        Returns:
            Assistant's response
        """
        request_data = {
            'model': model,
            'messages': messages,
            **kwargs
        }

        response = self.session.post(
            f'{self.base_url}/v1/chat/completions',
            json=request_data
        )
        response.raise_for_status()

        result = response.json()
        return result['choices'][0]['message']['content']

    def batch_inference(self,
                       model: str,
                       prompts: List[str],
                       max_tokens: int = 100,
                       webhook_url: Optional[str] = None) -> str:
        """
        Submit a batch of prompts for processing.

        Args:
            model: Model ID to use
            prompts: List of prompts to process
            max_tokens: Maximum tokens per response
            webhook_url: Optional webhook for completion notification

        Returns:
            Batch ID for tracking
        """
        requests_data = [
            {'id': f'req_{i}', 'prompt': prompt}
            for i, prompt in enumerate(prompts)
        ]

        request_data = {
            'model': model,
            'requests': requests_data,
            'max_tokens': max_tokens
        }

        if webhook_url:
            request_data['webhook_url'] = webhook_url

        response = self.session.post(
            f'{self.base_url}/batch',
            json=request_data
        )
        response.raise_for_status()

        return response.json()['batch_id']

    def get_batch_status(self, batch_id: str) -> Dict:
        """Get the status of a batch job."""
        response = self.session.get(f'{self.base_url}/batch/{batch_id}')
        response.raise_for_status()
        return response.json()

    def get_batch_results(self, batch_id: str) -> List[Dict]:
        """Get the results of a completed batch job."""
        response = self.session.get(f'{self.base_url}/batch/{batch_id}/results')
        response.raise_for_status()
        return response.json()['results']


class InfernoWebSocketClient:
    """WebSocket client for real-time streaming with Inferno."""

    def __init__(self, url: str = "ws://localhost:8080/ws", api_key: Optional[str] = None):
        """
        Initialize the WebSocket client.

        Args:
            url: WebSocket URL
            api_key: Optional API key for authentication
        """
        self.url = url
        self.api_key = api_key
        self.ws = None

    def connect(self):
        """Connect to the WebSocket server."""
        self.ws = websocket.WebSocketApp(
            self.url,
            on_open=self._on_open,
            on_message=self._on_message,
            on_error=self._on_error,
            on_close=self._on_close
        )

    def _on_open(self, ws):
        """Handle connection open."""
        print("WebSocket connection opened")

        # Send authentication if API key provided
        if self.api_key:
            auth_msg = {
                'type': 'auth',
                'token': self.api_key
            }
            ws.send(json.dumps(auth_msg))

    def _on_message(self, ws, message):
        """Handle incoming messages."""
        data = json.loads(message)

        if data['type'] == 'token':
            print(data['token'], end='', flush=True)
        elif data['type'] == 'complete':
            print("\n[Inference complete]")
        elif data['type'] == 'error':
            print(f"\n[Error: {data['message']}]")

    def _on_error(self, ws, error):
        """Handle errors."""
        print(f"WebSocket error: {error}")

    def _on_close(self, ws, close_status_code, close_msg):
        """Handle connection close."""
        print(f"WebSocket connection closed: {close_msg}")

    def send_inference(self, model: str, prompt: str, max_tokens: int = 100):
        """Send an inference request."""
        if not self.ws:
            raise Exception("Not connected to WebSocket")

        request = {
            'type': 'inference',
            'id': f'req_{int(time.time() * 1000)}',
            'model': model,
            'prompt': prompt,
            'max_tokens': max_tokens,
            'stream': True
        }

        self.ws.send(json.dumps(request))

    def run(self):
        """Run the WebSocket client."""
        self.ws.run_forever()


def main():
    """Example usage of the Inferno client."""

    # Initialize client
    client = InfernoClient(api_key="your_api_key_here")

    print("=== Inferno Python Client Example ===\n")

    # 1. Health check
    print("1. Health Check")
    health = client.health_check()
    print(f"   Status: {health['status']}")
    print(f"   Version: {health['version']}\n")

    # 2. List models
    print("2. Available Models")
    models = client.list_models()
    for model in models:
        print(f"   - {model['id']}: {model['name']} ({model['type']})")
    print()

    # 3. Load a model
    print("3. Loading Model")
    model_id = "llama-2-7b"
    # Uncomment to actually load:
    # result = client.load_model(model_id, gpu_layers=32)
    # print(f"   Model loaded: {result['status']}\n")

    # 4. Simple inference
    print("4. Simple Inference")
    prompt = "What is artificial intelligence?"
    print(f"   Prompt: {prompt}")
    # Uncomment to run inference:
    # response = client.inference(model_id, prompt, max_tokens=50)
    # print(f"   Response: {response}\n")

    # 5. Streaming inference
    print("5. Streaming Inference")
    prompt = "Tell me a short story about a robot"
    print(f"   Prompt: {prompt}")
    print("   Response: ", end="")
    # Uncomment to stream:
    # for token in client.stream_inference(model_id, prompt, max_tokens=100):
    #     print(token, end="", flush=True)
    # print("\n")

    # 6. Generate embeddings
    print("6. Text Embeddings")
    texts = ["Hello world", "How are you?", "Machine learning is fascinating"]
    print(f"   Texts: {texts}")
    # Uncomment to generate embeddings:
    # embeddings = client.embeddings(model_id, texts)
    # print(f"   Generated {len(embeddings)} embeddings")
    # print(f"   Embedding dimension: {len(embeddings[0])}\n")

    # 7. Chat completion (OpenAI compatible)
    print("7. Chat Completion")
    messages = [
        {"role": "system", "content": "You are a helpful assistant."},
        {"role": "user", "content": "What is the capital of France?"}
    ]
    print(f"   Messages: {len(messages)}")
    # Uncomment to run chat:
    # response = client.chat_completion(model_id, messages)
    # print(f"   Assistant: {response}\n")

    # 8. Batch processing
    print("8. Batch Processing")
    prompts = [
        "What is Python?",
        "Explain quantum computing",
        "How does photosynthesis work?"
    ]
    print(f"   Batch size: {len(prompts)}")
    # Uncomment to submit batch:
    # batch_id = client.batch_inference(model_id, prompts)
    # print(f"   Batch ID: {batch_id}")
    #
    # # Wait for completion
    # while True:
    #     status = client.get_batch_status(batch_id)
    #     if status['status'] == 'completed':
    #         break
    #     time.sleep(1)
    #
    # results = client.get_batch_results(batch_id)
    # print(f"   Completed: {len(results)} responses\n")

    # 9. WebSocket streaming (uncomment to test)
    print("9. WebSocket Streaming")
    print("   Connecting to WebSocket...")
    # ws_client = InfernoWebSocketClient(api_key="your_api_key_here")
    # ws_client.connect()
    # ws_client.send_inference(model_id, "Tell me a joke", max_tokens=50)
    # ws_client.run()

    print("\n=== Example Complete ===")


if __name__ == "__main__":
    main()