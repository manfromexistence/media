# Icon Search Integration Plan

## Problem Summary

WASM integration is **not viable** for 300K+ icons due to:
- Memory constraints (45KB+ JSON strings exceed WASM linear memory)
- serde_json parsing failures (consistent error at column 45264)
- Data transfer overhead (moving 300K icons from JS to WASM)

## Solution: Server-Side API

Use the native Rust `dx_icon_search` engine via HTTP API.

### Performance Achieved (Native Rust)
- Cold cache: 1.9ms
- Warm cache: 624µs
- Throughput: 98,783 searches/sec
- 10-25x faster than competitors

### Architecture

```
Browser (Next.js)
    ↓ HTTP request
API Route (/api/search)
    ↓ calls
Native Rust Engine (dx_icon_search)
    ↓ returns
JSON results
```

## Implementation Steps

### 1. Create Next.js API Route

**File**: `apps/www/app/api/search/route.ts`

```typescript
import { NextRequest, NextResponse } from 'next/server';
import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

export async function GET(request: NextRequest) {
  const searchParams = request.nextUrl.searchParams;
  const query = searchParams.get('q') || '';
  const limit = parseInt(searchParams.get('limit') || '100');
  
  if (!query) {
    return NextResponse.json({ results: [] });
  }
  
  try {
    // Call Rust CLI (assumes dx_icon_search binary is built)
    const { stdout } = await execAsync(
      `dx_icon_search search "${query}" --limit ${limit} --json`
    );
    
    const results = JSON.parse(stdout);
    return NextResponse.json({ results });
  } catch (error) {
    console.error('Search error:', error);
    return NextResponse.json({ error: 'Search failed' }, { status: 500 });
  }
}
```

### 2. Update Icon Loader

**File**: `apps/www/lib/icon-loader.ts`

```typescript
// Replace WASM search with API call
export async function searchIcons(query: string, limit: number = 100) {
  const response = await fetch(`/api/search?q=${encodeURIComponent(query)}&limit=${limit}`);
  const data = await response.json();
  return data.results;
}
```

### 3. Build Rust Binary

```bash
cd dx_icon_search
cargo build --release
# Binary at: target/release/dx_icon_search
```

### 4. Add CLI to dx_icon_search

**File**: `dx_icon_search/src/main.rs`

```rust
use clap::{Parser, Subcommand};
use dx_icon_search::{IconIndex, IconSearchEngine};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Search {
        query: String,
        #[arg(long, default_value = "100")]
        limit: usize,
        #[arg(long)]
        json: bool,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Search { query, limit, json } => {
            // Load index (cached in memory for subsequent searches)
            let index = IconIndex::load_from_file("icon_index.bin")?;
            let engine = IconSearchEngine::from_index(index)?;
            
            let results = engine.search(&query, limit);
            
            if json {
                println!("{}", serde_json::to_string(&results)?);
            } else {
                for result in results {
                    println!("{} ({}): {:.2}", result.icon.name, result.icon.pack, result.score);
                }
            }
            
            Ok(())
        }
    }
}
```

## Alternative: Keep Existing JS Search

If server-side API adds latency, keep the existing JavaScript search implementation. It's already working and the performance difference only matters at scale.

## Recommendation

**Use server-side API** for:
- Production deployments
- When search performance is critical
- When you can deploy the Rust binary alongside Next.js

**Keep JS search** for:
- Development
- Static exports
- Serverless deployments without custom binaries
