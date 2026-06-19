// GPU compute shader for parallel icon search
// Processes thousands of icons simultaneously across GPU cores

@group(0) @binding(0) var<storage, read> query: array<u32>;
@group(0) @binding(1) var<storage, read> icon_names: array<u32>;
@group(0) @binding(2) var<storage, read> offsets: array<u32>;
@group(0) @binding(3) var<storage, read_write> results: array<u32>;

// Workgroup size: 64 threads per workgroup (optimal for most GPUs)
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let icon_idx = global_id.x;
    
    // Bounds check
    if (icon_idx >= arrayLength(&offsets)) {
        return;
    }
    
    // Get icon name bounds
    let start = offsets[icon_idx];
    let end = select(
        offsets[icon_idx + 1u],
        arrayLength(&icon_names),
        icon_idx + 1u >= arrayLength(&offsets)
    );
    
    let icon_len = end - start;
    let query_len = arrayLength(&query);
    
    // Check if icon contains query (substring match)
    var score = 0u;
    
    // Exact match
    if (icon_len == query_len) {
        var matches = true;
        for (var i = 0u; i < query_len; i = i + 1u) {
            if (query[i] != icon_names[start + i]) {
                matches = false;
                break;
            }
        }
        if (matches) {
            score = 100u;
        }
    }
    
    // Prefix match (if not exact)
    if (score == 0u && icon_len >= query_len) {
        var matches = true;
        for (var i = 0u; i < query_len; i = i + 1u) {
            if (query[i] != icon_names[start + i]) {
                matches = false;
                break;
            }
        }
        if (matches) {
            score = 80u;
        }
    }
    
    // Substring match (if not prefix)
    if (score == 0u && icon_len > query_len) {
        for (var pos = 1u; pos < icon_len - query_len + 1u; pos = pos + 1u) {
            var matches = true;
            for (var i = 0u; i < query_len; i = i + 1u) {
                if (query[i] != icon_names[start + pos + i]) {
                    matches = false;
                    break;
                }
            }
            if (matches) {
                score = 50u;
                break;
            }
        }
    }
    
    results[icon_idx] = score;
}
