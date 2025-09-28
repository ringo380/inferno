#!/bin/bash

# GitHub Repository Organization Setup Script
# This script creates comprehensive labels for the Inferno project

REPO="ringo380/inferno"

echo "Setting up comprehensive GitHub labels for $REPO..."

# Function to create label if it doesn't exist
create_label() {
    local name="$1"
    local color="$2"
    local description="$3"

    echo "Creating label: $name"
    gh api --method POST repos/$REPO/labels \
        --field name="$name" \
        --field color="$color" \
        --field description="$description" \
        2>/dev/null || echo "Label '$name' may already exist or failed to create"
}

# Priority Labels (Red spectrum)
create_label "priority/critical" "B60205" "Critical priority - immediate attention required"
create_label "priority/high" "D93F0B" "High priority - should be addressed soon"
create_label "priority/medium" "FBCA04" "Medium priority - normal development cycle"
create_label "priority/low" "0E8A16" "Low priority - nice to have"

# Type Labels (Blue spectrum)
create_label "type/bug" "D73A4A" "Something isn't working as expected"
create_label "type/feature" "0075CA" "New feature or enhancement request"
create_label "type/improvement" "7057FF" "Improvement to existing functionality"
create_label "type/refactor" "5319E7" "Code refactoring with no functional changes"
create_label "type/documentation" "0366D6" "Documentation updates or additions"
create_label "type/testing" "1D76DB" "Testing related changes or additions"
create_label "type/performance" "0052CC" "Performance optimization or related"
create_label "type/security" "B60205" "Security related issue or enhancement"

# Component Labels (Green spectrum)
create_label "component/backend" "0E8A16" "Backend inference engines (GGUF, ONNX)"
create_label "component/api" "28A745" "HTTP API and WebSocket components"
create_label "component/cli" "34D058" "Command-line interface components"
create_label "component/tui" "40E869" "Terminal user interface components"
create_label "component/config" "85E89D" "Configuration management system"
create_label "component/models" "A2F2A2" "Model discovery and metadata"
create_label "component/metrics" "C3F7C3" "Performance monitoring and metrics"
create_label "component/cache" "D4F5D4" "Caching system and optimization"
create_label "component/batch" "E5F9E5" "Batch processing system"
create_label "component/io" "F0FDF0" "Input/output format handling"

# Area Labels (Purple spectrum)
create_label "area/infrastructure" "7B68EE" "Core infrastructure and foundation"
create_label "area/performance" "9370DB" "Performance and optimization"
create_label "area/enterprise" "8A2BE2" "Enterprise features and capabilities"
create_label "area/devops" "9932CC" "DevOps, deployment, and CI/CD"
create_label "area/security" "A020F0" "Security and authentication"
create_label "area/monitoring" "B83DBA" "Monitoring and observability"
create_label "area/distributed" "C71585" "Distributed computing and scaling"

# Backend Specific Labels (Orange spectrum)
create_label "backend/gguf" "FF8C00" "GGUF backend implementation"
create_label "backend/onnx" "FF7F00" "ONNX backend implementation"
create_label "backend/generic" "FF6347" "Generic backend functionality"

# Size/Effort Labels (Gray spectrum)
create_label "size/XS" "F1F8FF" "Extra small effort (< 2 hours)"
create_label "size/S" "E6E6FA" "Small effort (2-4 hours)"
create_label "size/M" "D3D3D3" "Medium effort (4-8 hours)"
create_label "size/L" "A9A9A9" "Large effort (1-2 days)"
create_label "size/XL" "808080" "Extra large effort (3-5 days)"
create_label "size/XXL" "696969" "Extra extra large effort (1+ weeks)"

# Status Labels (Cyan spectrum)
create_label "status/blocked" "FF0000" "Blocked by external dependency"
create_label "status/in-review" "20B2AA" "Currently under review"
create_label "status/needs-triage" "48D1CC" "Needs initial assessment and prioritization"
create_label "status/help-wanted" "7FFFD4" "Community help wanted"
create_label "status/good-first-issue" "00CED1" "Good for newcomers"

# Platform Labels (Brown spectrum)
create_label "platform/linux" "8B4513" "Linux platform specific"
create_label "platform/macos" "A0522D" "macOS platform specific"
create_label "platform/windows" "CD853F" "Windows platform specific"
create_label "platform/docker" "D2691E" "Docker containerization"

# Dependencies and Integration (Teal spectrum)
create_label "dependencies" "2F4F4F" "Dependency updates or issues"
create_label "integration/openai" "008B8B" "OpenAI API compatibility"
create_label "integration/websocket" "20B2AA" "WebSocket integration"
create_label "integration/prometheus" "48D1CC" "Prometheus metrics integration"

echo "GitHub labels setup completed!"
echo ""
echo "Next steps:"
echo "1. Review the created labels in your GitHub repository"
echo "2. Create milestones for version planning"
echo "3. Set up GitHub Projects for workflow management"
echo "4. Update issue templates to use these new labels"