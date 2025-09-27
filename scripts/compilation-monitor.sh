#!/bin/bash

# Compilation Progress Monitor for Inferno Dashboard
# Shows a clean progress bar and summary of what's happening

# Colors for better readability
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Function to draw progress bar
draw_progress_bar() {
    local current=$1
    local total=$2
    local width=50
    local percentage=$((current * 100 / total))
    local filled=$((current * width / total))
    local empty=$((width - filled))

    printf "["
    printf "%*s" $filled | tr ' ' '='
    printf "%*s" $empty | tr ' ' '-'
    printf "] %d/%d (%d%%)" $current $total $percentage
}

# Function to parse latest compilation output
parse_compilation_status() {
    local bash_id=$1
    local temp_file="/tmp/compilation_output.tmp"

    # Get the latest output
    claude-code bash-output "$bash_id" > "$temp_file" 2>/dev/null

    # Extract current compilation status
    local building_line=$(tail -20 "$temp_file" | grep -E "Building.*\[.*\].*/" | tail -1)

    if [[ $building_line =~ Building.*\[.*\]\ ([0-9]+)/([0-9]+): ]]; then
        current_package=${BASH_REMATCH[1]}
        total_packages=${BASH_REMATCH[2]}

        # Extract what's currently being compiled
        current_items=$(echo "$building_line" | sed -E 's/.*[0-9]+\/[0-9]+: (.+)\.\.\.[^a-zA-Z]*/\1/' | sed 's/\[K$//')

        echo "$current_package,$total_packages,$current_items"
    else
        echo "0,779,Initializing..."
    fi
}

# Function to estimate remaining time
estimate_time() {
    local current=$1
    local total=$2
    local start_time=$3

    if [ $current -gt 0 ]; then
        local elapsed=$(($(date +%s) - start_time))
        local rate=$(echo "scale=2; $current / $elapsed" | bc -l 2>/dev/null || echo "1")
        local remaining=$((total - current))
        local eta=$(echo "scale=0; $remaining / $rate" | bc -l 2>/dev/null || echo "300")

        local eta_min=$((eta / 60))
        local eta_sec=$((eta % 60))

        echo "${eta_min}m ${eta_sec}s"
    else
        echo "Calculating..."
    fi
}

# Main monitoring function
monitor_compilation() {
    local bash_id=${1:-"3d0f8a"}  # Default to the known bash ID
    local start_time=$(date +%s)

    echo -e "${BLUE}🦀 Inferno Dashboard Compilation Monitor${NC}"
    echo -e "${BLUE}==========================================${NC}"
    echo ""

    while true; do
        # Clear the previous output (but keep header)
        tput cup 4 0
        tput ed

        # Parse current status
        local status=$(parse_compilation_status "$bash_id")
        IFS=',' read -r current total items <<< "$status"

        # Display progress bar
        echo -e "${GREEN}Progress:${NC} $(draw_progress_bar $current $total)"
        echo ""

        # Display current phase
        echo -e "${YELLOW}Current Phase:${NC} Compiling Rust dependencies"
        echo -e "${YELLOW}Working on:${NC}   $items"
        echo ""

        # Time estimation
        local eta=$(estimate_time $current $total $start_time)
        local elapsed=$((($(date +%s) - start_time) / 60))
        echo -e "${BLUE}Elapsed:${NC}      ${elapsed} minutes"
        echo -e "${BLUE}ETA:${NC}          $eta"
        echo ""

        # Status summary
        echo -e "${GREEN}What's happening:${NC}"
        echo "  • Compiling Rust crates and dependencies"
        echo "  • Building Tauri backend with SQLite integration"
        echo "  • Processing $total total packages"
        echo "  • Next.js frontend is ready on port 3457"
        echo ""

        # Performance note
        if [ $current -gt 0 ]; then
            local percentage=$((current * 100 / total))
            if [ $percentage -lt 50 ]; then
                echo -e "${YELLOW}Note:${NC} First-time compilation includes downloading and building all dependencies"
            else
                echo -e "${GREEN}Note:${NC} More than halfway through! Backend compilation should complete soon"
            fi
        fi

        echo ""
        echo -e "${BLUE}Press Ctrl+C to exit monitor (compilation will continue)${NC}"

        # Wait before next update
        sleep 5
    done
}

# Check if bc is available for calculations
if ! command -v bc &> /dev/null; then
    echo "Installing bc for time calculations..."
    if command -v brew &> /dev/null; then
        brew install bc &> /dev/null
    fi
fi

# Start monitoring
echo "Starting compilation monitor..."
echo "Monitoring bash process: ${1:-3d0f8a}"
echo ""

monitor_compilation "$1"