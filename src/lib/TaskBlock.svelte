<script lang="ts">
  import { createEventDispatcher } from "svelte";

  interface Task {
    task_id: number;
    task_name: string;
    task_time_start: string;
    task_time_end: string;
    task_color: string;
  }

  interface Props {
    task: Task;
    hourHeight: number;
    startHour: number;
    currentTime: Date;
  }

  let { task, hourHeight, startHour, currentTime }: Props = $props();

  const dispatch = createEventDispatcher();

  let isHovered = $state(false);

  function parseTime(timeStr: string): { hours: number; minutes: number } {
    const [h, m] = timeStr.split(":");
    return { hours: parseInt(h, 10), minutes: parseInt(m, 10) };
  }

  function calculatePosition() {
    const start = parseTime(task.task_time_start);
    const end = parseTime(task.task_time_end);

    let startH = start.hours;
    let endH = end.hours;

    // Wrap-around logic: if time is before startHour, assume it's "next day" (past midnight)
    if (startH < startHour) startH += 24;
    // For end time, if it's smaller than start time OR smaller than startHour, add 24
    if (endH < startHour || endH < startH) endH += 24;

    const startMinutes = (startH - startHour) * 60 + start.minutes;
    const endMinutes = (endH - startHour) * 60 + end.minutes;
    const durationMinutes = endMinutes - startMinutes;

    const top = (startMinutes / 60) * hourHeight + 15;
    const height = (durationMinutes / 60) * hourHeight;

    return { top, height, durationMinutes };
  }

  function formatTimeRange(): string {
    return `${task.task_time_start} - ${task.task_time_end}`;
  }

  function formatDuration(minutes: number): string {
    const hrs = Math.floor(minutes / 60);
    const mins = minutes % 60;

    if (hrs == 1 && mins > 0) {
      return `${hrs} Stunde ${mins} Minuten`;
    } else if (hrs > 1 && mins > 0) {
      return `${hrs} Stunden ${mins} Minuten`;
    } else if (hrs == 1) {
      return `${hrs} Stunde`;
    } else if (hrs > 0) {
      return `${hrs} Stunden`;
    } else {
      return `${mins} Minuten`;
    }
  }

  // Function to determine if a color is dark
  function isDarkColor(hexColor: string): boolean {
    // Remove '#' if present
    const s = hexColor.startsWith("#") ? hexColor.slice(1) : hexColor;
    // Parse r, g, b values
    const r = parseInt(s.substring(0, 2), 16);
    const g = parseInt(s.substring(2, 4), 16);
    const b = parseInt(s.substring(4, 6), 16);
    // Calculate luminance (YIQ formula)
    const yiq = (r * 299 + g * 587 + b * 114) / 1000;
    // Return true for dark colors, false for light colors
    return yiq < 128;
  }

  let position = $derived(calculatePosition());

  function getRemainingMinutes(now: Date): number | null {
    const nowH = now.getHours();
    const nowM = now.getMinutes();
    const nowMins = nowH * 60 + nowM;

    const [endH, endM] = task.task_time_end.split(":").map(Number);
    let endMins = endH * 60 + endM;

    const [startH, startM] = task.task_time_start.split(":").map(Number);
    let startMins = startH * 60 + startM;

    // Wrap adjustments
    if (startMins < startHour * 60) startMins += 24 * 60;
    if (endMins < startHour * 60 || endMins < startMins) endMins += 24 * 60;

    // We also need to wrap 'now' if it's "next day" compared to startHour
    let currentMins = nowMins;
    if (currentMins < startHour * 60) currentMins += 24 * 60;

    if (currentMins >= startMins && currentMins < endMins) {
      return endMins - currentMins;
    }
    return null;
  }

  let remaining = $derived(getRemainingMinutes(currentTime));
</script>

<div
  class="task-block"
  style="
    top: {position.top}px;
    height: {position.height}px;
    background-color: {task.task_color};
    color: {isDarkColor(task.task_color) ? 'white' : 'black'};
  "
  class:small={position.height < 65}
  class:micro={position.durationMinutes < 45}
  onclick={() => dispatch("edit", task)}
>
  <div class="task-content">
    <div class="task-header">
      <span class="task-name">{task.task_name}</span>
    </div>
    <span class="task-time">{formatTimeRange()}</span>
    <span class="task-duration">
      {formatDuration(position.durationMinutes)}
      {#if remaining !== null}
        ({formatDuration(remaining)} übrig)
      {/if}
    </span>
  </div>
</div>

<style>
  .task-block {
    position: absolute;
    left: 50px;
    right: 10px;
    border-radius: 4px;
    padding: 8px 12px;
    box-sizing: border-box;
    cursor: pointer;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    transition: filter 0.2s;
    cursor: pointer;
  }

  .task-block:hover {
    filter: brightness(0.95);
    z-index: 5;
  }

  .task-block.small {
    padding: 2px 8px;
  }

  .task-content {
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex: 1;
    min-height: 0;
  }

  .task-name {
    font-size: 13px;
    font-weight: 500;
    color: #333;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  /* Micro task styles (under 45 mins) */
  .task-block.micro {
    min-height: 20px;
    padding: 2px 6px; /* Reduced vertical padding but keep top alignment */
    flex-direction: row;
    align-items: flex-start; /* Align contents to the top */
    overflow: hidden;
    z-index: 10;
  }

  .task-block.micro .task-content {
    flex-direction: row;
    gap: 8px;
    align-items: center; /* Keep items aligned with each other text-wise */
    flex-wrap: wrap; /* allow wrap if really needed? No, user wants horizontal. */
  }

  .task-block.micro .task-header {
    margin-bottom: 0;
    flex-shrink: 0;
    max-width: none;
    display: flex; /* Ensure header is a flex container too if needed, or just block */
    align-items: center;
  }

  .task-block.micro .task-name {
    font-weight: bold;
    font-size: 13px;
    line-height: normal; /* Reset line height */
    margin-top: 1px; /* visual tweak if needed, or remove */
  }

  /* Show duration for micro tasks */
  .task-block.micro .task-duration {
    display: inline-block;
    font-size: 11px;
    opacity: 0.8;
    line-height: normal;
    margin-top: 1px;
  }

  .task-block.micro .task-time {
    line-height: normal;
    margin-top: 1px;
  }

  .task-time {
    font-size: 11px;
    color: #555;
  }

  .task-duration {
    font-size: 11px;
    color: #555;
    margin-top: 2px;
  }

  .task-content.compact {
    flex-direction: row;
    align-items: baseline;
    gap: 8px;
    flex: 0;
  }

  .task-content.compact .task-duration {
    margin-top: 0;
  }

  .task-block.small .edit-btn {
    top: 2px;
    right: 2px;
    padding: 2px;
    font-size: 12px;
  }

  .edit-btn {
    position: absolute;
    top: 8px;
    right: 8px;
    background: none;
    border: none;
    cursor: pointer;
    font-size: 14px;
    opacity: 0;
    transition: opacity 0.2s;
    padding: 4px;
  }

  .edit-btn.visible {
    opacity: 1;
  }
</style>
