<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import AddTaskModal from "$lib/AddTaskModal.svelte";
  import EditTaskModal from "$lib/EditTaskModal.svelte";
  import TaskBlock from "$lib/TaskBlock.svelte";

  interface Task {
    task_id: number;
    task_name: string;
    task_time_start: string;
    task_time_end: string;
    task_color: string;
  }

  let tasks = $state<Task[]>([]);
  let currentTime = $state(new Date());
  let showAddModal = $state(false);
  let showEditModal = $state(false);
  let selectedTask = $state<Task | null>(null);
  let isHeaderHovered = $state(false);
  let calendarContainer = $state<HTMLElement | null>(null);

  const startHour = 6;
  const endHour = 30;
  const hourHeight = 100;

  const hours = Array.from(
    { length: endHour - startHour },
    (_, i) => startHour + i,
  );

  onMount(() => {
    loadTasks();

    if (calendarContainer) {
      const containerHeight = calendarContainer.clientHeight;
      const scrollTarget = timeIndicatorPosition - containerHeight / 2;
      calendarContainer.scrollTo({ top: scrollTarget, behavior: "instant" });
    }

    // Update current time every minute
    const interval = setInterval(() => {
      currentTime = new Date();
    }, 60000);

    return () => clearInterval(interval);
  });

  async function loadTasks() {
    try {
      tasks = await invoke("get_all_tasks");
    } catch (e) {
      console.error("Failed to load tasks:", e);
    }
  }

  async function handleDeleteAll() {
    try {
      await invoke("delete_all_tasks");
      tasks = [];
    } catch (e) {
      console.error("Failed to delete tasks:", e);
    }
  }

  function handleTaskCreated(event: CustomEvent<Task>) {
    tasks = [...tasks, event.detail].sort((a, b) =>
      a.task_time_start.localeCompare(b.task_time_start),
    );
  }

  function handleTaskUpdated(event: CustomEvent<Task>) {
    tasks = tasks
      .map((t) => (t.task_id === event.detail.task_id ? event.detail : t))
      .sort((a, b) => a.task_time_start.localeCompare(b.task_time_start));
  }

  function handleTaskDeleted(event: CustomEvent<number>) {
    tasks = tasks.filter((t) => t.task_id !== event.detail);
  }

  function openEditModal(event: CustomEvent<Task>) {
    selectedTask = event.detail;
    showEditModal = true;
  }

  function formatTime(date: Date): string {
    return date.toLocaleTimeString("de-DE", {
      hour: "2-digit",
      minute: "2-digit",
    });
  }

  function formatDate(date: Date): string {
    return date.toLocaleDateString("de-DE", {
      day: "2-digit",
      month: "2-digit",
      year: "numeric",
    });
  }

  function formatHour(hour: number): string {
    const displayHour = hour % 24;
    return `${displayHour.toString().padStart(2, "0")}:00`;
  }

  function getCurrentTimePosition(): number {
    let hours = currentTime.getHours();
    const minutes = currentTime.getMinutes();

    // Logic to handle wrapping: if current time is < startHour (e.g. 1 AM), treat it as next day (25)
    if (hours < startHour) {
      hours += 24;
    }

    const totalMinutes = (hours - startHour) * 60 + minutes;
    return (totalMinutes / 60) * hourHeight;
  }

  let timeIndicatorPosition = $derived(getCurrentTimePosition());

  function handleKeydown(event: KeyboardEvent) {
    if (showAddModal || showEditModal) return;
    if (event.key === "Enter") {
      showAddModal = true;
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<main class="app">
  <header
    class="header"
    onmouseenter={() => (isHeaderHovered = true)}
    onmouseleave={() => (isHeaderHovered = false)}
  >
    <div class="time-display">
      <span class="time">{formatTime(currentTime)}</span>
      <span class="date">{formatDate(currentTime)}</span>
    </div>

    <div class="header-actions" class:visible={isHeaderHovered}>
      <button
        class="icon-btn"
        onclick={handleDeleteAll}
        title="Alle Aufgaben löschen"
      >
        🗑
      </button>
      <button
        class="icon-btn"
        onclick={() => (showAddModal = true)}
        title="Aufgabe hinzufügen"
      >
        +
      </button>
    </div>
  </header>

  <div class="calendar-container" bind:this={calendarContainer}>
    <div class="calendar">
      {#each hours as hour}
        <div class="hour-row" style="height: {hourHeight}px;">
          <span class="hour-label">{formatHour(hour)}</span>
          <div class="hour-line"></div>
        </div>
      {/each}

      <!-- Current time indicator -->
      {#if timeIndicatorPosition >= 0 && timeIndicatorPosition <= (endHour - startHour) * hourHeight}
        <div
          class="time-indicator"
          style="top: {timeIndicatorPosition + 20}px;"
        ></div>
      {/if}

      <!-- Task blocks -->
      {#each tasks as task (task.task_id)}
        <TaskBlock
          {task}
          {hourHeight}
          {startHour}
          {currentTime}
          on:edit={openEditModal}
        />
      {/each}
    </div>
  </div>

  <AddTaskModal
    bind:show={showAddModal}
    existingTasks={tasks}
    on:taskCreated={handleTaskCreated}
  />

  <EditTaskModal
    bind:show={showEditModal}
    task={selectedTask}
    existingTasks={tasks}
    on:taskUpdated={handleTaskUpdated}
    on:taskDeleted={(e) => handleTaskDeleted(e)}
    on:tasksShifted={loadTasks}
  />
</main>

<style>
  :global(*) {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
  }

  :global(body) {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Oxygen,
      Ubuntu, sans-serif;
    background: #f5f5f5;
    overflow: hidden;
  }

  .app {
    height: 100vh;
    display: flex;
    flex-direction: column;
    background: white;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    background: transparent;
    min-height: 48px;
  }

  .time-display {
    display: flex;
    gap: 12px;
    align-items: baseline;
  }

  .time {
    font-size: 16px;
    font-weight: 500;
    color: #333;
  }

  .date {
    font-size: 16px;
    color: #333;
    font-weight: 500;
  }

  .header-actions {
    display: flex;
    gap: 8px;
    opacity: 0;
    transition: opacity 0.2s;
  }

  .header-actions.visible {
    opacity: 1;
  }

  .icon-btn {
    background: none;
    border: none;
    font-size: 20px;
    cursor: pointer;
    padding: 4px 8px;
    color: #555;
    transition: color 0.15s;
  }

  .icon-btn:hover {
    color: #333;
  }

  .calendar-container {
    flex: 1;
    overflow-y: auto;
    position: relative;
  }

  .calendar {
    position: relative;
    min-height: 100%;
  }

  .hour-row {
    display: flex;
    align-items: flex-start;
    position: relative;
  }

  .hour-label {
    width: 45px;
    padding: 4px 8px;
    font-size: 12px;
    color: #888;
    text-align: right;
    flex-shrink: 0;
    background: white;
    border-radius: 4px;
    margin: 4px;
    margin-left: 4px;
  }

  .hour-line {
    flex: 1;
    margin-top: 12px;
  }

  .time-indicator {
    position: absolute;
    left: 0;
    right: 0;
    height: 2px;
    background: #d32f2f;
    z-index: 20;
    pointer-events: none;
  }
</style>
