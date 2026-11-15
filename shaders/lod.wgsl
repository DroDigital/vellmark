// Levels of detail, chosen by how large a thing is on screen: the share of the view its
// bounding sphere fills. The culling pass and the surface shaders both call this, so
// they agree on every level and every crossfade.

// Below each share, a thing is drawn one level coarser.
const LOD_SHARES: array<f32, 3> = array<f32, 3>(0.09, 0.04, 0.018);
// Over the last fifth above each share, the two levels are crossfaded.
const LOD_BAND: f32 = 0.2;

// x: the level. y: how far into its crossfade with the next level, 0 to 1; 0 where
// there is none.
fn lod_of(radius: f32, distance: f32, levels: u32) -> vec2<f32> {
    let share = radius / max(distance, 0.01);
    var level = 0u;
    for (var k = 0u; k + 1u < levels && k < 3u; k++) {
        if share < LOD_SHARES[k] {
            level = k + 1u;
        }
    }
    var fade = 0.0;
    if level + 1u < levels && level < 3u {
        let edge = LOD_SHARES[level];
        fade = clamp((edge * (1.0 + LOD_BAND) - share) / (edge * LOD_BAND), 0.0, 1.0);
    }
    return vec2<f32>(f32(level), fade);
}
