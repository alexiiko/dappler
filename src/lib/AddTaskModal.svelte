<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { createEventDispatcher } from "svelte";

  interface Props {
    show: boolean;
    existingTasks?: {
      task_time_start: string;
      task_time_end: string;
      task_id: number;
    }[];
  }

  let { show = $bindable(false), existingTasks = [] }: Props = $props();

  const dispatch = createEventDispatcher();

  let taskName = $state("");
  let timeStart = $state("");
  let timeEnd = $state("");
  let selectedColor = $state("#B8A4D9");
  let errorMessage = $state("");

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
    // Remove all non-digits
    const digits = t.replace(/\D/g, "");

    // If it's 3 digits (e.g. 930) -> 09:30
    if (digits.length === 3) {
      return `0${digits[0]}:${digits.slice(1)}`;
    }
    // If it's 4 digits (e.g. 1330) -> 13:30
    if (digits.length === 4) {
      return `${digits.slice(0, 2)}:${digits.slice(2)}`;
    }
    // Return original if no match (validateTime will catch it)
    return t;
  }

  function checkOverlap(start: string, end: string): boolean {
    for (const task of existingTasks) {
      if (start < task.task_time_end && task.task_time_start < end) {
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
      errorMessage = "Aufgabenname ist nötig!";
      return;
    }

    if (!timeStart || !timeEnd) {
      errorMessage = "Start und Endzeit sind erforderlich!";
      return;
    }

    // Normalize inputs (e.g. 1300 -> 13:00)
    timeStart = normalizeTime(timeStart);
    timeEnd = normalizeTime(timeEnd);

    if (!validateTime(timeStart) || !validateTime(timeEnd)) {
      errorMessage = "Zeiten müssen im Format HH:MM sein (z.B. 13:00)!";
      return;
    }

    if (timeStart >= timeEnd) {
      errorMessage = "Endzeit muss nach Startzeit kommen!";
      return;
    }

    if (checkOverlap(timeStart, timeEnd)) {
      errorMessage = "Aufgabe überlappt sich mit einer bestehenden Aufgabe!";
      return;
    }

    try {
      const newTask = await invoke("create_task", {
        taskName: taskName.trim(),
        taskTimeStart: timeStart,
        taskTimeEnd: timeEnd,
        taskColor: selectedColor,
      });
      dispatch("taskCreated", newTask);
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
  }
  let taskNameInput: HTMLInputElement;

  $effect(() => {
    if (show && taskNameInput) {
      taskNameInput.focus();
    }
  });

  function handleColorKeydown(e: KeyboardEvent, color: string) {
    if (e.key === "Enter") {
      selectedColor = color;
      handleAddTask();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if show}
  <div class="modal-overlay" onclick={closeModal}>
    <div class="modal" onclick={(e) => e.stopPropagation()}>
      <div class="modal-header">
        <h2>Aufgabe hinzufügen</h2>
        <button class="close-btn" onclick={closeModal}>×</button>
      </div>

      <div class="modal-body">
        <div class="form-group">
          <label for="task-name">Aufgabenname:</label>
          <input
            type="text"
            id="task-name"
            bind:this={taskNameInput}
            bind:value={taskName}
            placeholder=""
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
        <button class="submit-btn" onclick={handleSubmit}>Hinzufügen</button>
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
    padding: 0; /* Reset button padding */
    appearance: none; /* Reset button appearance */
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
    z-index: 1; /* Ensure outline is visible */
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

  input[type="time"]::-webkit-calendar-picker-indicator {
    display: none;
  }
</style>
