# Phase 8: Instance Detail Screen

## Status: Complete

## Summary

Implemented the instance detail screen at `/instances/[id]` with full instance information display, inline name editing, lifecycle controls, and CLI setup instructions.

## Deliverables

### 1. SectionHeader Component
- **File:** `src/lib/components/SectionHeader.svelte`
- Simple section header component with title and optional description
- Used for organizing the detail page into logical sections

### 2. Instance Detail Route
- **File:** `src/routes/instances/[id]/+page.svelte`
- Full instance detail page with:
  - Back navigation to instances list
  - Instance header with editable name, status badge, and version
  - Action buttons (Open Gateway, Start/Stop/Restart)
  - Details section with key-value layout:
    - Gateway URL (copyable)
    - Gateway token (masked with reveal toggle, copyable)
    - Bridge port
    - Network access mode
    - Timezone
    - Config path (copyable)
    - Workspace path (copyable)
    - Container ID (copyable)
  - Actions section (Edit Settings, Rebuild placeholders)
  - CLI Setup section with docker compose commands
  - Danger zone with Delete Instance placeholder

### 3. Instance Store Update
- **File:** `src/lib/stores/instances.svelte.ts`
- Added `initialized` getter to the store export

### 4. InstanceCard Navigation
- **File:** `src/lib/components/InstanceCard.svelte`
- Made entire card clickable to navigate to detail page
- Click handlers on buttons prevent event bubbling to avoid double navigation
- Added keyboard accessibility (Enter key support)

## Features Implemented

### Inline Name Editing
- Click on instance name to enter edit mode
- Save on Enter or blur
- Cancel on Escape
- Calls `rename_instance` command on save
- Updates local store after successful rename

### Token Masking
- Gateway token is masked by default (shows 8 bullet characters)
- Eye icon toggle to reveal/hide the token
- Copy button works regardless of mask state

### Lifecycle Controls
- Open Gateway button (running instances) - opens gateway URL in browser
- Start button (stopped instances)
- Stop button (running instances)
- Restart button (running, stopped, or error instances)
- All buttons show loading states during operations

### Docker Status Integration
- Instance status respects Docker availability
- Shows "Docker Not Running" state when Docker is unavailable
- Disables actions when Docker is not running

### Error Handling
- Loading state while fetching instance
- Error display if instance not found
- Error message display for instances in error state

## Manual Test Checklist

- [ ] Click an instance in the list -> detail screen loads
- [ ] All info fields display correctly
- [ ] Copy buttons work for all copyable fields
- [ ] Token is masked by default
- [ ] Token reveal toggle works
- [ ] Copy button copies actual token (not masked version)
- [ ] Rename works (click name, edit, save on Enter)
- [ ] Cancel rename works (Escape key)
- [ ] Start/Stop/Restart buttons work
- [ ] Open Gateway button opens browser
- [ ] Back button returns to list
- [ ] CLI setup commands display correctly
- [ ] Status updates when instance state changes

## Files Changed

- `src/lib/components/SectionHeader.svelte` (new)
- `src/routes/instances/[id]/+page.svelte` (rewritten)
- `src/lib/components/InstanceCard.svelte` (updated - clickable card)
- `src/lib/stores/instances.svelte.ts` (updated - added initialized getter)

## Notes

- Edit Settings and Rebuild buttons are placeholders for Phase 9
- Delete Instance button in danger zone is placeholder for Phase 9
- CLI setup section provides docker compose commands for manual provider/channel setup
