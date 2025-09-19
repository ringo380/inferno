# Inferno Dashboard

A modern, responsive web dashboard for the Inferno AI/ML platform. Built with Next.js, React, TypeScript, and Tailwind CSS.

## Features

### 🏠 **Dashboard Overview**
- Real-time system metrics and performance monitoring
- Quick actions for common tasks
- System resource usage visualization
- Recent activity feed

### 🧠 **Model Management**
- Upload and manage AI models (GGUF, ONNX, PyTorch, SafeTensors)
- Model quantization with multiple compression options
- Batch model operations
- Model performance analytics

### ⚡ **Inference Console**
- Real-time model testing with streaming responses
- Configurable inference parameters
- Preset configurations for different use cases
- Inference history tracking

### 📊 **Performance Monitoring**
- Real-time metrics visualization
- System resource monitoring
- Model performance tracking
- Custom dashboards and alerts

### 🔧 **Batch Jobs**
- Schedule and manage batch processing jobs
- Job queue monitoring
- Cron-based scheduling
- Progress tracking and logs

### ⚙️ **Settings & Configuration**
- System configuration management
- User preferences
- API key management
- Security settings

## Technology Stack

- **Frontend Framework**: Next.js 14 with App Router
- **UI Library**: React 18 with TypeScript
- **Styling**: Tailwind CSS with custom design system
- **State Management**: React Query for server state
- **Charts**: Recharts for data visualization
- **Icons**: Lucide React
- **Forms**: React Hook Form with Zod validation
- **Theme**: Next Themes for dark/light mode
- **Real-time**: Socket.IO for WebSocket connections

## Installation

### Prerequisites

- Node.js 18+
- npm or yarn
- Inferno AI/ML backend running

### Setup

1. **Clone the repository**
   ```bash
   cd inferno/dashboard
   ```

2. **Install dependencies**
   ```bash
   npm install
   # or
   yarn install
   ```

3. **Environment Configuration**
   Create a `.env.local` file:
   ```env
   INFERNO_API_URL=http://localhost:8080
   INFERNO_WS_URL=ws://localhost:8080
   NEXT_PUBLIC_APP_URL=http://localhost:3000
   ```

4. **Start the development server**
   ```bash
   npm run dev
   # or
   yarn dev
   ```

5. **Open the dashboard**
   Navigate to [http://localhost:3000](http://localhost:3000)

## Production Deployment

### Build for Production

```bash
npm run build
npm start
```

### Docker Deployment

```dockerfile
FROM node:18-alpine

WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production

COPY . .
RUN npm run build

EXPOSE 3000
CMD ["npm", "start"]
```

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `INFERNO_API_URL` | Inferno backend API URL | `http://localhost:8080` |
| `INFERNO_WS_URL` | WebSocket connection URL | `ws://localhost:8080` |
| `NEXT_PUBLIC_APP_URL` | Public dashboard URL | `http://localhost:3000` |

## Development

### Project Structure

```
src/
├── app/                 # Next.js App Router pages
├── components/          # React components
│   ├── ui/             # Base UI components
│   ├── layout/         # Layout components
│   ├── dashboard/      # Dashboard-specific components
│   ├── models/         # Model management components
│   └── inference/      # Inference console components
├── lib/                # Utility functions
├── types/              # TypeScript type definitions
├── hooks/              # Custom React hooks
└── api/                # API client functions
```

### Key Components

- **MainLayout**: Core application layout with header and sidebar
- **ModelManagement**: Full-featured model management interface
- **InferenceConsole**: Real-time model testing and parameter tuning
- **DashboardOverview**: System metrics and quick actions
- **ThemeProvider**: Dark/light mode theme system

### Available Scripts

- `npm run dev` - Start development server
- `npm run build` - Build for production
- `npm run start` - Start production server
- `npm run lint` - Run ESLint
- `npm run test` - Run Jest tests
- `npm run storybook` - Start Storybook development

### Testing

```bash
# Unit tests
npm run test

# E2E tests
npm run test:e2e

# Watch mode
npm run test:watch
```

## API Integration

The dashboard integrates with the Inferno backend through REST APIs and WebSocket connections:

### REST Endpoints
- `GET /api/models` - List available models
- `POST /api/models/upload` - Upload new model
- `POST /api/inference` - Run inference
- `GET /api/metrics` - System metrics
- `GET /api/jobs` - Batch jobs

### WebSocket Events
- `metrics` - Real-time system metrics
- `inference_stream` - Streaming inference responses
- `job_update` - Batch job status updates
- `system_status` - System status changes

## Customization

### Theme Customization

Edit `tailwind.config.ts` to customize colors, fonts, and spacing:

```typescript
theme: {
  extend: {
    colors: {
      primary: {
        // Your custom primary colors
      }
    }
  }
}
```

### Adding New Pages

1. Create page component in `src/app/[page]/page.tsx`
2. Add navigation item to `src/components/layout/sidebar.tsx`
3. Update types in `src/types/inferno.ts` if needed

### Custom Components

Follow the established patterns:
- Use TypeScript interfaces
- Implement dark mode support
- Include accessibility features
- Add proper error handling

## Performance

### Optimization Features

- **Code Splitting**: Automatic route-based code splitting
- **Image Optimization**: Next.js Image component
- **Bundle Analysis**: Webpack Bundle Analyzer
- **Caching**: React Query for API caching
- **Lazy Loading**: Component-level lazy loading

### Performance Monitoring

The dashboard includes built-in performance monitoring:
- Real-time metrics collection
- API response time tracking
- Component render performance
- Memory usage monitoring

## Security

### Security Features

- **Input Validation**: Zod schema validation
- **XSS Protection**: Sanitized user inputs
- **CSRF Protection**: Next.js built-in protection
- **Secure Headers**: Security-focused HTTP headers
- **API Authentication**: JWT token support

### Best Practices

- Environment variables for sensitive data
- Secure API communication
- Input sanitization
- Error boundary implementation

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass
6. Submit a pull request

### Code Style

- Use TypeScript for all components
- Follow ESLint configuration
- Use Prettier for code formatting
- Include JSDoc comments for complex functions
- Maintain accessibility standards

## Troubleshooting

### Common Issues

**Dashboard won't load**
- Check that Inferno backend is running
- Verify API URLs in environment variables
- Check browser console for errors

**WebSocket connection fails**
- Ensure WebSocket URL is correct
- Check firewall settings
- Verify backend WebSocket server is running

**Build failures**
- Clear `node_modules` and reinstall
- Check Node.js version compatibility
- Verify all environment variables are set

### Support

For issues and questions:
- Check the [Inferno documentation](../README.md)
- Review the [troubleshooting guide](../TROUBLESHOOTING_GUIDE.md)
- Submit issues on GitHub

## License

Licensed under the same terms as the Inferno project (MIT/Apache 2.0).