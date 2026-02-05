# Visual Quality Review Criteria

> Target: **Dark Reign / Supreme Commander / Total Annihilation** quality
>
> Created for headless playtesting screenshot review.

## Readability

- [ ] Units distinguishable at max zoom-out
- [ ] Faction colors clearly different
- [ ] Health bars readable
- [ ] Selection indicators visible
- [ ] Attack animations clear
- [ ] Unit types recognizable by silhouette alone
- [ ] Important units (heroes, siege) stand out

## Faction Identity

### Continuity (Tech/Preservation)

- [ ] Blue-silver palette consistent across all units
- [ ] Smooth, curved forms (shell-like aesthetic)
- [ ] Glowing blue accents visible (eyes, vents, weapon tips)
- [ ] Tech/preservation motifs clear
- [ ] Preservation pods/stasis elements on buildings
- [ ] Clean, minimalist aesthetic maintained

### Collegium (Knowledge/Geometric)

- [ ] Gold-bronze palette consistent across all units
- [ ] Angular, crystalline forms
- [ ] Floating geometric elements present (orbiting cubes, pyramids)
- [ ] Visible energy conduits on structures
- [ ] Archive/library motifs on buildings
- [ ] Knowledge/scholar aesthetic clear

## Animation Quality

### Movement

- [ ] Idle animations present and subtle
- [ ] Move animations smooth (no foot sliding)
- [ ] Turning animations natural
- [ ] Speed appropriate to unit type

### Combat

- [ ] Attack animations impactful
- [ ] Wind-up telegraphs intent
- [ ] Hit reactions visible
- [ ] Death animations clear and informative
- [ ] No units disappearing instantly

### Buildings

- [ ] Construction animations visible
- [ ] Production indicators active
- [ ] Damage states progressive
- [ ] Destruction satisfying

## Effects

### Weapons

- [ ] Muzzle flashes visible but not overwhelming
- [ ] Projectile trails readable
- [ ] Impact effects clear
- [ ] Different weapon types distinguishable

### Explosions & Destruction

- [ ] Explosion effects appropriate scale
- [ ] Debris doesn't obscure gameplay
- [ ] Wreckage fades appropriately

### UI Elements

- [ ] Selection circles distinct
- [ ] Order indicators visible
- [ ] Range indicators readable
- [ ] Minimap icons clear

## Consistency

### Scale

- [ ] Similar units same apparent size
- [ ] Infantry < Vehicles < Buildings progression clear
- [ ] No units look out of place size-wise

### Lighting

- [ ] Shadow directions consistent
- [ ] Lighting coherent with environment
- [ ] Faction glows don't clash

### Ground Contact

- [ ] Units appear grounded (no floating)
- [ ] Shadows connect units to terrain
- [ ] Buildings have clear footprints

## Zoom Levels

### Max Zoom-out (Strategic View)

- [ ] Unit icons/dots visible
- [ ] Faction colors dominant
- [ ] Army compositions discernible
- [ ] Buildings clearly marked

### Default Zoom (Gameplay View)

- [ ] All above criteria pass
- [ ] Comfortable for extended play
- [ ] No visual fatigue

### Max Zoom-in (Detail View)

- [ ] Textures hold up
- [ ] Animation detail visible
- [ ] No visible artifacts

## Combat Scenarios

### 1v1 Unit Fight

- [ ] Both units clearly visible
- [ ] Attack direction obvious
- [ ] Health changes trackable

### Squad Combat (5-10 units)

- [ ] Individual units still readable
- [ ] Composition balance visible
- [ ] Focus fire target identifiable

### Large Battle (20+ units)

- [ ] Faction areas distinguishable
- [ ] Battle lines readable
- [ ] Critical units (artillery, healers) findable

## Known Issues Checklist

Use this section to track recurring issues:

| Issue | Severity | Status | Notes |
| --- | --- | --- | --- |
| _Example: Infantry blends with terrain_ | Medium | Open | Needs outline or shadow |
| | | | |
| | | | |

## Review Workflow

1. **Load Screenshot Manifest**: Open `results/{batch_id}/manifest.json`
2. **Review Each Screenshot**: Check against relevant criteria above
3. **Log Issues**: Add to Known Issues table
4. **Rate Quality**: 1-5 scale for each category
5. **Generate Report**: Summarize findings for art team

## Automated Checks

The following can be automated via image analysis:

### Silhouette Test

- Threshold image to binary
- Count distinct blobs
- Compare to expected unit count
- **Pass**: Detected count matches expected (±10%)

### Color Distinction Test

- Sample pixels from each faction's units
- Calculate LAB color space distance
- **Pass**: Distance > 50 units (clearly different)

### Brightness Test

- Calculate average brightness per faction
- **Pass**: Neither faction too dark (<0.2) or too bright (>0.8)

### Overlap Test

- Check no units share >20% of their bounding boxes
- **Pass**: Overlap percentage < 10% average

---

## Changelog

| Date        | Reviewer | Changes                          |
| ----------- | -------- | -------------------------------- |
| 2026-02-03  | AI       | Initial criteria from design doc |
