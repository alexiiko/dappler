<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { createEventDispatcher, untrack } from "svelte";

  interface Task {
    task_id: number;
    task_name: string;
    task_time_start: string;
    task_time_end: string;
    task_color: string;
  }

  interface Props {
    show: boolean;
    task: Task | null;
    existingTasks?: Task[];
  }

  let {
    show = $bindable(false),
    task = null,
    existingTasks = [],
  }: Props = $props();

  const dispatch = createEventDispatcher();

  let taskName = $state("");
  let timeStart = $state("");
  let timeEnd = $state("");
  let selectedColor = $state("#B8A4D9");
  let errorMessage = $state("");
  let currentTaskId = $state(-1);

  const colors = [
    "#FFADAD", // Pastel Red
    "#FFD6A5", // Pastel Orange
    "#FDFFB6", // Pastel Yellow
    "#CAFFBF", // Pastel Green
    "#9BF6FF", // Pastel Cyan
    "#A0C4FF", // Pastel Blue
    "#BDB2FF", // Pastel Purple
    "#FFC6FF", // Pastel Pink
    "#A0E8AF", // Mint
    "#98E2E6", // Aqua
    "#ffbfd3", // Salmon
    "#E2F0CB", // Pale Lime
    "#C8E7F5", // Sky
    "#FBEB93", // Lemon
    "#D0CDE1", // Lavender Gray
    "#E6E6FA", // Lavender
    "#E8DFF5", // Pearl Purple
    "#FCE1E4", // Piggy Pink
    "#FCF4DD", // Pale Gold
    "#DDEDEA", // Pale Azure
  ];

  function normalizeTime(t: string): string {
    const digits = t.replace(/\D/g, "");
    if (digits.length === 3) return `0${digits[0]}:${digits.slice(1)}`;
    if (digits.length === 4) return `${digits.slice(0, 2)}:${digits.slice(2)}`;
    return t;
  }

  function timeToMinutes(t: string): number {
    const [h, m] = t.split(":").map(Number);
    return (h || 0) * 60 + (m || 0);
  }

  let canShift = $state(false);

  $effect(() => {
    if (task && show) {
      // Initialization logic: check if we switched tasks or opened modal
      if (task.task_id !== currentTaskId) {
        untrack(() => {
          currentTaskId = task.task_id;
          taskName = task.task_name;
          timeStart = task.task_time_start;
          timeEnd = task.task_time_end;
          selectedColor = task.task_color;
        });
      }

      // Shift Calculation Logic (Reactive to timeEnd changes)
      // We rely on timeEnd being up to date (either from init above or user input)
      const nEnd = normalizeTime(timeEnd);
      if (/^([0-1]?[0-9]|2[0-3]):[0-5][0-9]$/.test(nEnd)) {
        const oldMins = timeToMinutes(task.task_time_end);
        let newMins = timeToMinutes(nEnd);

        // Check basic difference
        // We can use the same logic as backend or just check string equality if we trust normalization?
        // But we already have minutes.
        // If newMins != oldMins (taking wrap into account is irrelevant for "is changed", but relevant for direction)
        // We just want to enable if "logic end time changed".
        const changed = newMins !== oldMins; // Simple check is enough for enabling button.

        // Check for following tasks
        const startOffset = 6 * 60;
        // Ensure refEndMins logic matches how we identify "following":
        const refEndMins = oldMins < startOffset ? oldMins + 24 * 60 : oldMins;

        const hasFollowing = existingTasks.some((t) => {
          if (t.task_id === task.task_id) return false;
          let tStart = timeToMinutes(t.task_time_start);
          if (tStart < startOffset) tStart += 24 * 60;
          return tStart >= refEndMins;
        });

        canShift = changed && hasFollowing;
      } else {
        canShift = false;
      }
    } else {
      // Reset ID when closed so it re-inits next time even if same task clicked
      currentTaskId = -1;
    }
  });

  async function handleShift() {
    if (!canShift || !task) return;

    // Normalize logic
    timeStart = normalizeTime(timeStart);
    timeEnd = normalizeTime(timeEnd);

    // Validation (reuse handleSubmit logic partially or just minimal)
    if (!validateTime(timeStart) || !validateTime(timeEnd)) return;

    // We assume overlap check is skipped/handled by the shift logic (which creates space)
    // But update_task_with_shift doesn't check for NEW overlaps of the current task itself.
    // It assumes user knows what they are doing or we rely on visual.
    // But let's basic check.
    // Actually, prompt says: "To update this i need to change both tasks so that they dont overlap."
    // "When the button is pressed, every following task if shifted".
    // This implies we FORCE the shift to make room.

    try {
      const updatedTask = await invoke("update_task_with_shift", {
        id: task.task_id,
        taskName: taskName.trim(),
        taskTimeStart: timeStart,
        taskTimeEnd: timeEnd,
        taskColor: selectedColor,
        originalEnd: task.task_time_end,
      });
      // We need to refresh ALL tasks since many changed.
      // Dispatch specific event or just generic update?
      // "taskUpdated" usually updates one.
      // We should dispatch a "refreshAll" or similar?
      // Or just taskUpdated and let parent reload? Parent reloads on taskUpdated?
      // In +page.svelte: handleTaskUpdated updates ARRAY.
      // It won't know about other shifted tasks.
      // We should dispatch a new event type "tasksShifted"?
      // Or just reload.
      dispatch("tasksShifted");
      closeModal();
    } catch (e) {
      errorMessage = String(e);
    }
  }

  // ... (existing effects/functions) ...

  function checkOverlap(start: string, end: string): boolean {
    for (const t of existingTasks) {
      if (task && t.task_id === task.task_id) continue;
      if (start < t.task_time_end && t.task_time_start < end) {
        return true;
      }
    }
    return false;
  }

  function validateTime(time: string): boolean {
    const regex = /^([0-1]?[0-9]|2[0-3]):[0-5][0-9]$/;
    return regex.test(time);
  }

  async function handleSubmit() {
    errorMessage = "";

    if (!taskName.trim()) {
      errorMessage = "Task name is required";
      return;
    }

    if (!timeStart || !timeEnd) {
      errorMessage = "Start and end times are required";
      return;
    }

    // Normalize inputs (e.g. 1300 -> 13:00)
    timeStart = normalizeTime(timeStart);
    timeEnd = normalizeTime(timeEnd);

    if (!validateTime(timeStart) || !validateTime(timeEnd)) {
      errorMessage = "Times must be in HH:MM format (e.g. 13:00)";
      return;
    }

    if (timeStart >= timeEnd) {
      errorMessage = "End time must be after start time";
      return;
    }

    if (checkOverlap(timeStart, timeEnd)) {
      errorMessage = "Task overlaps with an existing task";
      return;
    }

    if (!task) return;

    try {
      const updatedTask = await invoke("update_task", {
        id: task.task_id,
        taskName: taskName.trim(),
        taskTimeStart: timeStart,
        taskTimeEnd: timeEnd,
        taskColor: selectedColor,
      });
      dispatch("taskUpdated", updatedTask);
      closeModal();
    } catch (e) {
      errorMessage = String(e);
    }
  }

  async function handleDelete() {
    if (!task) return;

    try {
      await invoke("delete_task", { id: task.task_id });
      dispatch("taskDeleted", task.task_id);
      closeModal();
    } catch (e) {
      errorMessage = String(e);
    }
  }

  function closeModal() {
    taskName = "";
    timeStart = "";
    timeEnd = "";
    selectedColor = "#B8A4D9";
    errorMessage = "";
    show = false;
  }
  function handleKeydown(event: KeyboardEvent) {
    if (!show) return;
    if (event.key === "Escape") {
      closeModal();
    }
    if (event.key === "Enter") {
      handleSubmit();
    }
    // Delete shortcut (Backspace or Delete) - only if not in an input
    if (event.key === "Backspace" || event.key === "Delete") {
      const target = event.target as HTMLElement;
      const isInput =
        target.tagName === "INPUT" || target.tagName === "TEXTAREA";
      if (!isInput) {
        handleDelete();
      }
    }
  }

  function handleColorKeydown(e: KeyboardEvent, color: string) {
    if (e.key === "Enter") {
      selectedColor = color;
      handleSubmit();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if show && task}
  <div class="modal-overlay" onclick={closeModal}>
    <div class="modal" onclick={(e) => e.stopPropagation()}>
      <div class="modal-header">
        <h2>Aufgabe bearbeiten</h2>
        <div class="header-buttons">
          <button class="delete-btn" onclick={handleDelete} title="Delete task"
            >🗑</button
          >
          <button class="close-btn" onclick={closeModal}>×</button>
        </div>
      </div>

      <div class="modal-body">
        <div class="form-group">
          <label for="task-name-edit">Aufgabenname:</label>
          <input
            type="text"
            id="task-name-edit"
            bind:value={taskName}
            autocomplete="off"
            autofocus
          />
        </div>

        <div class="form-group">
          <label>Intervall:</label>
          <div class="time-inputs">
            <input type="text" placeholder="" bind:value={timeStart} />
            <span>-</span>
            <input type="text" placeholder="" bind:value={timeEnd} />
          </div>
        </div>

        <div class="color-group">
          <label>Farbe:</label>
          <div class="color-grid">
            {#each colors as color}
              <button
                class="color-swatch"
                style="background-color: {color}"
                class:selected={selectedColor === color}
                onclick={() => (selectedColor = color)}
                onkeydown={(e) => handleColorKeydown(e, color)}
                aria-label="Select color {color}"
              ></button>
            {/each}
          </div>
        </div>

        {#if errorMessage}
          <p class="error">{errorMessage}</p>
        {/if}
      </div>

      <div class="modal-footer">
        <button
          class="shift-btn"
          disabled={!canShift}
          onclick={handleShift}
          title="Alle nachfolgenden Aufgaben verschieben"
        >
          Verschieben
        </button>
        <button class="submit-btn" onclick={handleSubmit}
          >Aufgabe bearbeiten</button
        >
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal {
    background: white;
    border-radius: 8px;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
    width: 320px;
    max-width: 90%;
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 20px;
  }

  .modal-header h2 {
    margin: 0;
    font-size: 18px;
    font-weight: 500;
    color: #333;
  }

  .header-buttons {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .delete-btn {
    background: none;
    border: none;
    font-size: 18px;
    cursor: pointer;
    color: #666;
    padding: 4px;
    line-height: 1;
  }

  .delete-btn:hover {
    color: #d32f2f;
  }

  .close-btn {
    background: none;
    border: none;
    font-size: 28px;
    cursor: pointer;
    color: #666;
    padding: 0;
    line-height: 1;
  }

  .close-btn:hover {
    color: #333;
  }

  .modal-body {
    padding: 20px;
  }

  .form-group {
    margin-bottom: 20px;
  }

  .form-group label {
    display: block;
    margin-bottom: 8px;
    font-size: 14px;
    color: #555;
  }

  .form-group input[type="text"] {
    width: 100%;
    padding: 10px 12px;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-size: 14px;
    box-sizing: border-box;
  }

  .time-inputs {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .time-inputs input {
    flex: 1;
    padding: 10px 12px;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-size: 14px;
  }

  .time-inputs span {
    color: #666;
  }

  .color-grid {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: 10px;
  }

  .color-swatch {
    width: 40px;
    height: 40px;
    border: 2px solid transparent;
    border-radius: 6px;
    cursor: pointer;
    transition:
      transform 0.15s,
      border-color 0.15s;
    padding: 0;
    appearance: none;
    outline: none;
  }

  .color-swatch:hover {
    transform: scale(1.1);
  }

  .color-swatch.selected {
    border-color: #333;
    transform: scale(1.1);
  }

  .color-swatch:focus {
    outline: 2px solid #555;
    outline-offset: 2px;
    z-index: 1;
  }

  .error {
    color: #d32f2f;
    font-size: 13px;
    margin: 0;
  }

  .modal-footer {
    padding: 16px 20px;
    display: flex;
    justify-content: flex-end;
  }

  .submit-btn {
    padding: 10px 20px;
    background: white;
    border: 1px solid #ddd;
    border-radius: 4px;
    cursor: pointer;
    font-size: 14px;
    transition: background 0.15s;
  }

  .submit-btn:hover {
    background: #f5f5f5;
  }

  .shift-btn {
    padding: 10px 20px;
    background: white;
    border: 1px solid #ddd;
    border-radius: 4px;
    cursor: pointer;
    font-size: 14px;
    color: #555;
    margin-right: 10px;
    transition: all 0.15s;
  }

  .shift-btn:hover:not(:disabled) {
    background: #f0f0f0;
    border-color: #ccc;
  }

  .shift-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
    background: #fafafa;
  }

  input[type="time"]::-webkit-calendar-picker-indicator {
    display: none;
  }
</style>
