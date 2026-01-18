# Swarm Symphony

A boid flocking simulation demonstrating mdhavers language features, creating emergent music from collective motion.

## Overview

Swarm Symphony simulates hundreds of autonomous agents (boids) exhibiting flocking behavior through three simple rules:
- **Separation**: Avoid crowding neighbors
- **Alignment**: Steer toward average heading of neighbors
- **Cohesion**: Steer toward average position of neighbors

The simulation maps swarm behavior to procedural audio, creating a living soundscape that reflects the collective state of the flock.

## Features

- **3D Boid Simulation** - Classic Reynolds flocking with predators, attractors, and repulsors
- **Isometric 2D Rendering** - 3D world projected to 2D for efficient raylib rendering
- **Procedural Audio** - Position and velocity mapped to pitch, volume, and pan
- **Coherence-Based Harmony** - Swarm organization affects musical scale selection
- **P2P Networking** - Distributed swarms across multiple nodes
- **Headless Mode** - Pipe-based control for automated testing

## Quick Start

```bash
# Run with graphics (requires graphics feature)
SWARM_GFX=1 mdhavers run swarm_symphony/swarm.braw

# Run headless with 100 boids for 1000 ticks
SWARM_HEADLESS=1 SWARM_BOIDS=100 SWARM_TICKS=1000 mdhavers run swarm_symphony/swarm.braw

# Run with JSON output for parsing
SWARM_HEADLESS=1 SWARM_JSON=1 SWARM_TICKS=10 mdhavers run swarm_symphony/swarm.braw

# Load commands from batch file
SWARM_HEADLESS=1 SWARM_BATCH=commands.txt mdhavers run swarm_symphony/swarm.braw

# Run with audio (requires audio feature)
SWARM_AUDIO=1 mdhavers run swarm_symphony/swarm.braw

# Run with networking
SWARM_NET=1 SWARM_PORT=9000 mdhavers run swarm_symphony/swarm.braw
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `SWARM_HEADLESS` | Run without interactive input | - |
| `SWARM_BOIDS` | Initial number of boids | 50 |
| `SWARM_TICKS` | Max ticks (0 = unlimited) | 0 |
| `SWARM_VERBOSE` | Enable verbose logging | - |
| `SWARM_JSON` | Output state as JSON | - |
| `SWARM_BATCH` | Path to batch command file | - |
| `SWARM_GFX` | Enable graphics | - |
| `SWARM_AUDIO` | Enable audio | - |
| `SWARM_NET` | Enable networking | - |
| `SWARM_PORT` | Network port | 9000 |
| `SWARM_HOST` | Bind address | 0.0.0.0 |
| `SWARM_PEER` | Initial peer (host:port) | - |

## Commands

Interactive and batch commands:

```
spawn <kind> [x y z]   - Spawn agent (boid/predator/attractor/repulsor/player)
remove <id>            - Remove agent by ID
move <id> <dx dy dz>   - Set agent velocity
rule <name> <value>    - Modify simulation rule
pause                  - Toggle pause
tick [n]               - Advance n ticks (default 1)
status                 - Print simulation status
batch <file>           - Load commands from file
quit                   - Exit simulation
help                   - Show command help
```

## Simulation Rules

Adjustable via `rule` command:

| Rule | Description | Default |
|------|-------------|---------|
| `separation_radius` | Distance for separation | 2.0 |
| `separation_weight` | Separation force strength | 1.5 |
| `alignment_radius` | Distance for alignment | 5.0 |
| `alignment_weight` | Alignment force strength | 1.0 |
| `cohesion_radius` | Distance for cohesion | 5.0 |
| `cohesion_weight` | Cohesion force strength | 1.0 |
| `fear_radius` | Predator avoidance distance | 10.0 |
| `fear_weight` | Fear force strength | 3.0 |
| `max_speed` | Maximum agent speed | 2.0 |
| `max_force` | Maximum steering force | 0.1 |

## Architecture

```
swarm_symphony/
├── math.braw        - Vector math and spatial hashing
├── simulation.braw  - Agent system and boid behaviors
├── command.braw     - Command parser and I/O
├── audio.braw       - Procedural audio system
├── graphics.braw    - 2D isometric rendering
├── network.braw     - P2P networking
├── main.braw        - Unit tests only
├── run.braw         - Minimal headless runner
└── swarm.braw       - Full integrated application
```

## Module Dependencies

```
swarm.braw
├── command.braw
│   └── simulation.braw
│       └── math.braw
├── audio.braw
├── graphics.braw
└── network.braw
```

## Example Batch File

```
# setup.txt - Example batch commands
spawn predator 0 0 0
spawn attractor 20 0 0
spawn attractor -20 0 0
spawn repulsor 0 20 0
rule max_speed 3.0
rule separation_weight 2.0
tick 100
status
```

## Audio Mapping

The audio system maps simulation state to sound:

- **Position → Pitch**: X-axis position maps to scale degree
- **Velocity → Volume**: Faster agents are louder
- **Position → Pan**: X-axis position maps to stereo pan
- **Coherence → Scale**: High coherence uses pentatonic, low uses chromatic

## Network Protocol

UDP-based JSON messages:

```json
{
  "v": 1,           // Protocol version
  "t": 3,           // Message type (AGENT_UPDATE)
  "id": "node_...", // Sender node ID
  "ts": 1234567890, // Timestamp
  "p": { ... }      // Payload
}
```

Message types:
- 1: PING
- 2: PONG
- 3: AGENT_UPDATE
- 4: AGENT_SPAWN
- 5: AGENT_REMOVE
- 6: WORLD_STATE
- 7: SYNC_REQUEST

## Performance

- Spatial hashing provides O(1) neighbor queries
- Fixed timestep (60Hz) ensures determinism
- Quantized audio (16th notes at 120 BPM) reduces audio load
- Agent limit of 2000 for smooth performance

## Credits

Inspired by:
- Craig Reynolds' Boids (1986)
- Emergence and swarm intelligence
- Generative music systems

Written in mdhavers - the Scots programming language.
