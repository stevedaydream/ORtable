<template>
  <div class="flex flex-col h-full">

    <!-- Toolbar -->
    <div class="flex items-center gap-2 px-3 py-2 border-b border-gray-700 shrink-0">
      <span class="text-xs font-semibold text-gray-400 shrink-0">房間總覽</span>
      <input
        type="date"
        v-model="todayStr"
        class="bg-gray-800 border border-gray-700 rounded px-2 py-0.5 text-xs text-blue-400 font-mono outline-none focus:border-blue-500 hover:border-gray-500 cursor-pointer transition-colors [color-scheme:dark]"
        @change="onDateChange"
      />
      <span class="text-[10px] text-gray-600 shrink-0">({{ todayWeekday }})</span>
      <button
        v-if="!isToday"
        class="text-[10px] text-blue-400 hover:text-blue-200 px-1.5 py-0.5 rounded border border-blue-800 hover:border-blue-600 hover:bg-blue-900/30 transition-colors shrink-0"
        @click="jumpToToday"
      >今天</button>
      <div class="flex-1" />
      <template v-if="!editMode">
        <button
          class="text-xs text-gray-500 hover:text-gray-300 flex items-center gap-1.5 px-2 py-1 rounded hover:bg-gray-800 transition-colors shrink-0"
          @click="reload"
        >
          <i class="fa-solid fa-arrows-rotate text-[10px]"></i>手動重整
        </button>
        <button
          class="text-xs text-blue-400 hover:text-blue-200 flex items-center gap-1.5 px-2 py-1 rounded border border-blue-800 hover:border-blue-600 hover:bg-blue-900/30 transition-colors shrink-0"
          @click="enterEditMode"
        >
          <i class="fa-solid fa-pen-to-square text-[10px]"></i>編輯時段
        </button>
      </template>
      <template v-else>
        <span class="text-[10px] text-blue-400 flex items-center gap-1 shrink-0">
          <i class="fa-solid fa-pen-to-square animate-pulse"></i>時段編輯中
        </span>
        <button
          class="text-xs text-gray-400 hover:text-gray-200 px-2 py-1 rounded border border-gray-600 hover:border-gray-500 hover:bg-gray-800 transition-colors shrink-0"
          @click="cancelEditMode"
        >取消</button>
        <button
          class="text-xs text-white font-bold px-3 py-1 rounded bg-blue-700 hover:bg-blue-600 transition-colors shrink-0"
          @click="saveEditMode"
        >儲存</button>
      </template>
    </div>

    <!-- Edit mode banner -->
    <div v-if="editMode" class="shrink-0 border-b px-3 py-1.5 flex items-center gap-2 text-xs transition-colors"
         :class="selectedShift ? 'bg-blue-900/60 border-blue-600/60 text-blue-200' : 'bg-blue-950/70 border-blue-700/50 text-blue-300'">
      <template v-if="!selectedShift">
        <i class="fa-solid fa-hand-pointer text-blue-400 text-[10px]"></i>
        點擊科別色塊選取，再點擊目標房間欄移動
      </template>
      <template v-else>
        <i class="fa-solid fa-circle-dot text-blue-300 text-[10px] animate-pulse"></i>
        已選取
        <span class="font-bold text-white px-1.5 py-0.5 rounded text-[10px]" :class="deptColor(selectedShift.dept)">
          {{ selectedShift.dept }}
        </span>
        <span class="text-blue-400 font-mono text-[10px]">{{ fmtTs(selectedShift.start_time) }}–{{ fmtTs(selectedShift.end_time) }}</span>
        → 點擊目標房間欄移入
        <button class="ml-2 text-gray-500 hover:text-gray-300 text-[10px]" @click="selectedShift = null">取消選取</button>
      </template>
      <div class="flex-1" />
      <span v-if="stagedShifts.length > 0" class="text-amber-400 text-[10px]">
        <i class="fa-solid fa-triangle-exclamation mr-1"></i>{{ stagedShifts.length }} 個時段在暫存區
      </span>
    </div>

    <!-- Empty state -->
    <div v-if="rooms.length === 0" class="flex-1 flex flex-col items-center justify-center text-center px-6 text-gray-600">
      <i class="fa-solid fa-door-open text-4xl mb-3 opacity-30"></i>
      <div class="text-sm font-medium">尚未設定手術室</div>
      <div class="text-xs mt-1">請至「設定 → 房間管理」新增房間，或匯入月排班 Excel</div>
    </div>

    <!-- Timeline：單一捲動容器 -->
    <div v-else class="flex-1 overflow-auto" ref="scrollEl">
      <div class="flex min-w-max">

        <!-- ── 時間軸欄 (sticky left) ──────────────────────────── -->
        <div class="w-14 shrink-0 border-r border-gray-700 sticky left-0 z-20 bg-gray-900 select-none">
          <div :style="{ height: HEADER_H + 'px' }" class="sticky top-0 z-30 bg-gray-900 border-b border-gray-700"></div>
          <div class="relative" :style="{ height: TOTAL_PX + 'px' }">
            <div
              v-for="(h, i) in HOUR_LABELS" :key="h.label"
              class="absolute right-1 text-[10px] text-gray-500 leading-none"
              :style="{ top: (i === 0 ? 2 : h.y - 7) + 'px' }"
            >{{ h.label }}</div>
            <div
              v-if="nowY !== null"
              class="absolute left-0 right-0 border-t border-red-500/70"
              :style="{ top: nowY + 'px' }"
            />
          </div>
        </div>

        <!-- ── 房間欄 ───────────────────────────────────────────── -->
        <div
          v-for="room in rooms" :key="room.id"
          class="w-40 shrink-0 border-r border-gray-700/50"
        >
          <!-- 標頭（sticky top）兩行：房間名稱 + 人員 -->
          <div
            :style="{ height: HEADER_H + 'px' }"
            class="sticky top-0 z-10 border-b flex flex-col overflow-hidden relative"
            :class="[
              !room.is_active ? 'bg-gray-900 border-gray-800' :
                room.is_backup ? 'bg-orange-900/50 border-orange-700/50' : 'bg-gray-800 border-gray-700',
              room.is_active && dragOverRoom === room.name ? 'ring-2 ring-inset ring-blue-500/60' : '',
            ]"
          >
            <!-- 第一行：房間名稱 + 啟用開關 -->
            <div class="flex items-center gap-1 px-2 shrink-0" style="height: 32px;">
              <i v-if="room.is_backup" class="fa-solid fa-circle-exclamation text-orange-400 text-[10px] shrink-0"></i>
              <span class="flex-1 text-xs font-semibold truncate"
                :class="!room.is_active ? 'text-gray-600' : room.is_backup ? 'text-orange-300' : 'text-gray-200'"
              >{{ room.name }}</span>
              <span v-if="room.is_backup && room.is_active" class="text-[9px] text-orange-500 shrink-0">備用</span>
              <span v-if="!room.is_active" class="text-[9px] text-gray-600 shrink-0">停用</span>
              <!-- Switch -->
              <button
                class="relative inline-flex h-3.5 w-6 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 focus:outline-none"
                :class="room.is_active ? 'bg-green-500' : 'bg-gray-600'"
                :title="room.is_active ? '點擊停用此房間' : '點擊啟用此房間'"
                @click.stop="toggleRoomActive(room)"
              >
                <span
                  class="pointer-events-none inline-block h-2.5 w-2.5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out"
                  :class="room.is_active ? 'translate-x-2.5' : 'translate-x-0'"
                />
              </button>
            </div>
            <!-- 第二行：今日人員名牌 -->
            <div class="flex flex-wrap gap-0.5 px-1.5 overflow-hidden" style="height: 36px;">
              <span
                v-for="a in assignmentsForRoom(room.name)" :key="a.id"
                class="group inline-flex items-center gap-0.5 px-1 py-0.5 rounded text-[9px] leading-tight whitespace-nowrap cursor-grab active:cursor-grabbing relative"
                :class="roleBadgeClass(a.role)"
                :title="`${a.role} — ${a.staff_name} (可拖回人員區 或 右鍵取消)`"
                draggable="true"
                @dragstart="onAssignmentDragStart($event, a)"
                @dragend="onAssignmentDragEnd"
                @contextmenu.prevent="onAssignmentRightClick($event, a)"
              >
                <span class="font-bold">{{ roleShort(a.role) }}</span>
                <span>{{ a.staff_name }}</span>
              </span>
              <span v-if="assignmentsForRoom(room.name).length === 0" class="text-[9px] text-gray-700 leading-tight pt-0.5">
                未排班
              </span>
            </div>
            <!-- 第三行：刀數摘要 -->
            <div class="flex items-center gap-1 px-2 shrink-0" style="height: 24px; border-top: 1px solid rgba(75,85,99,0.3);">
              <template v-if="roomSummary(room.name).active + roomSummary(room.name).scheduled + roomSummary(room.name).recovery > 0">
                <template v-if="roomSummary(room.name).active > 0">
                  <span class="w-1.5 h-1.5 rounded-full bg-green-400 animate-pulse shrink-0"></span>
                  <span class="text-[9px] text-green-300 mr-1">{{ roomSummary(room.name).active }}</span>
                </template>
                <template v-if="roomSummary(room.name).scheduled > 0">
                  <span class="w-1.5 h-1.5 rounded-full bg-blue-400 shrink-0"></span>
                  <span class="text-[9px] text-blue-300 mr-1">{{ roomSummary(room.name).scheduled }}</span>
                </template>
                <template v-if="roomSummary(room.name).recovery > 0">
                  <span class="w-1.5 h-1.5 rounded-full bg-purple-400 shrink-0"></span>
                  <span class="text-[9px] text-purple-300">{{ roomSummary(room.name).recovery }}</span>
                </template>
              </template>
              <span v-else class="text-[9px] text-gray-700">空刀</span>
            </div>
            <!-- 拖曳時 header overlay（確保 header 區域也可接受 drop） -->
            <div
              v-if="room.is_active && isDragging"
              class="absolute inset-0 z-20"
              @dragover.prevent="dragOverRoom = room.name"
              @dragleave="onDragLeave(room.name)"
              @drop.prevent="onDrop($event, room.name)"
            />
          </div>

          <!-- 時間軸主體 -->
          <div
            class="relative transition-colors"
            :style="{ height: TOTAL_PX + 'px' }"
            :class="room.is_active && dragOverRoom === room.name ? 'bg-blue-500/10' : ''"
          >
            <!-- 整點格線 -->
            <div
              v-for="h in HOUR_LABELS" :key="h.label"
              class="absolute left-0 right-0 border-t border-gray-700/30"
              :style="{ top: h.y + 'px' }"
            />
            
            <!-- 科別色塊 -->
            <div
              v-for="shift in shiftsForRoom(room.name)" :key="shift.id"
              class="absolute left-1 right-1 rounded px-1.5 overflow-hidden select-none transition-all z-[9]"
              :class="[
                shift.is_emergency ? 'bg-red-900/80 border border-red-500' : deptColor(shift.dept),
                editMode
                  ? selectedShift?.id === shift.id
                    ? 'opacity-100 ring-2 ring-white cursor-pointer scale-[1.03]'
                    : 'opacity-75 cursor-pointer hover:opacity-95 hover:ring-1 hover:ring-white/40'
                  : shift.is_emergency ? 'opacity-90 pointer-events-none' : 'opacity-50 pointer-events-none',
              ]"
              :style="blockStyle(shift)"
              :title="editMode ? (selectedShift?.id === shift.id ? '已選取，點擊目標房間欄移動' : `點擊選取 ${shift.dept}，右鍵快速指派`) : undefined"
              @click.stop="editMode && onShiftBlockClick(shift)"
              @contextmenu.prevent="editMode && onShiftRightClick($event, shift)"
            >
              <div class="flex items-center gap-1">
                <span v-if="shift.is_emergency" class="shrink-0 text-[9px] font-bold text-red-300 bg-red-700/60 px-1 rounded leading-tight">急診房</span>
                <div class="truncate text-[10px] font-medium leading-tight pt-0.5" :class="shift.is_emergency ? 'text-red-100' : ''">{{ shift.dept }}</div>
              </div>
              <div class="truncate text-[9px] opacity-70 leading-none">{{ fmtTs(shift.start_time) }}–{{ fmtTs(shift.end_time) }}</div>
            </div>

            <!-- 病患卡片渲染 (Scheduled Tasks) -->
            <PatientCard
              v-for="task in scheduledTasksForRoom(room.name)" :key="task.id"
              class="absolute left-1 right-1 cursor-pointer transition-opacity"
              :class="editMode ? 'opacity-10 pointer-events-none' : ''"
              :task="task"
              mode="timeline"
              :style="taskBlockStyle(task)"
              @contextmenu="onTaskRightClick($event, task)"
              @click="detailTaskId = task.id"
            />

            <!-- Edit Mode：點選移入 overlay -->
            <div
              v-if="editMode && selectedShift"
              class="absolute inset-0 z-[8] transition-colors"
              :class="selectedShift.room_name === room.name
                ? 'bg-amber-500/10 cursor-not-allowed'
                : 'bg-blue-500/8 hover:bg-blue-500/20 cursor-pointer'"
              @click="onRoomColumnClick(room.name)"
            >
              <div class="absolute inset-0 flex items-center justify-center pointer-events-none">
                <span
                  v-if="selectedShift.room_name !== room.name"
                  class="text-[10px] text-blue-300 bg-blue-950/90 border border-blue-700/60 px-2 py-1 rounded shadow"
                >點擊移入此房間</span>
                <span
                  v-else
                  class="text-[10px] text-amber-400 bg-amber-950/90 border border-amber-700/60 px-2 py-1 rounded shadow"
                >目前位置</span>
              </div>
            </div>

            <!-- 現在線 -->
            <div
              v-if="nowY !== null"
              class="absolute left-0 right-0 border-t border-red-500/50 z-10"
              :style="{ top: nowY + 'px' }"
            />

            <!-- 停用 overlay（蓋住整欄，阻擋所有互動） -->
            <div
              v-if="!room.is_active"
              class="absolute inset-0 z-30 bg-gray-900/70 flex items-start justify-center pt-8 pointer-events-auto"
            >
              <span class="text-[10px] text-gray-600 bg-gray-900/80 px-2 py-1 rounded select-none">
                <i class="fa-solid fa-ban mr-1"></i>房間已停用
              </span>
            </div>

            <!-- 拖曳時透明 overlay -->
            <div
              v-if="room.is_active && isDragging && dragType !== 'assignment'"
              class="absolute inset-0 z-20"
              @dragover.prevent="dragOverRoom = room.name"
              @dragleave="onDragLeave(room.name)"
              @drop.prevent="onDrop($event, room.name)"
            >
              <div
                v-if="dragOverRoom === room.name"
                class="absolute inset-0 bg-blue-500/10 flex items-center justify-center pointer-events-none"
              >
                <span class="text-[10px] text-blue-300 bg-blue-950/90 px-2 py-1 rounded">
                  {{ dragType === "task" ? "排入此房間" : dragType === "shift" ? "移動至此房間" : "指派人員" }}
                </span>
              </div>
            </div>
          </div>
        </div>

      </div>
    </div>

    <!-- Conflict Staging Area -->
    <div
      v-show="stagedShifts.length > 0 || (isDragging && dragType === 'shift')"
      class="shrink-0 border-t border-gray-700 transition-colors"
      :class="stagingDragOver ? 'bg-amber-500/10' : ''"
      @dragover.prevent="stagingDragOver = true"
      @dragleave="stagingDragOver = false"
      @drop.prevent="onDropToStaging"
    >
      <div class="flex items-center gap-2 px-3 py-1.5">
        <i class="fa-solid fa-triangle-exclamation text-amber-400 text-[10px]"></i>
        <span class="text-[10px] font-semibold text-amber-400">衝突暫存區</span>
        <span class="text-[10px] text-gray-600 ml-1">{{ stagedShifts.length }}</span>
        <div class="flex-1" />
        <span class="text-[9px] text-gray-600">可從房間拖入暫存，或從此拖回房間</span>
      </div>
      <div class="px-3 pb-2 min-h-[28px] flex flex-wrap gap-1.5">
        <span
          v-for="s in stagedShifts" :key="s.id"
          class="inline-flex items-center gap-1 px-2 py-0.5 rounded text-[10px] cursor-grab active:cursor-grabbing select-none"
          :class="deptColor(s.dept)"
          draggable="true"
          @dragstart="onStagedShiftDragStart($event, s)"
          @dragend="onShiftDragEnd"
          :title="`${s.dept} ${fmtTs(s.start_time)}–${fmtTs(s.end_time)}（拖回房間）`"
        >
          {{ s.dept }} {{ fmtTs(s.start_time) }}–{{ fmtTs(s.end_time) }}
          <button class="ml-0.5 hover:text-red-300 leading-none" @click.stop="removeStaged(s.id)">×</button>
        </span>
        <span v-if="stagedShifts.length === 0 && isDragging && dragType === 'shift'" class="text-[10px] text-amber-500/50 italic self-center">
          放開以暫存此時段
        </span>
      </div>
    </div>

    <!-- Role picker overlay (drag-and-drop) -->
    <Teleport to="body">
      <div
        v-if="dropState"
        class="fixed inset-0 bg-black/60 z-[9995] flex items-center justify-center"
        @mousedown.self="dropState = null"
      >
        <div class="bg-gray-800 border border-gray-600 rounded-xl shadow-2xl p-4 min-w-[240px]">
          <div class="text-sm font-semibold text-gray-200 mb-1">
            指派 <span class="text-blue-300">{{ dropState.staffName }}</span>
          </div>
          <div class="text-[10px] text-gray-500 mb-3">
            房間：<span class="text-gray-300">{{ dropState.roomName }}</span>
          </div>
          <div class="text-[10px] text-gray-400 mb-1.5 font-semibold">選擇角色</div>
          <div class="grid grid-cols-3 gap-1.5 mb-3">
            <button
              v-for="r in ROLES" :key="r.value"
              class="py-1.5 rounded text-[10px] font-bold border-2 transition-all"
              :class="r.inactiveClass"
              @click="selectRoleAndConfirm(r.value)"
            >{{ r.label }}</button>
          </div>
          <div class="text-center">
            <button class="text-[10px] text-gray-500 hover:text-gray-300" @click="dropState = null">取消</button>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- Room Disable Confirmation Modal -->
    <Teleport to="body">
      <div
        v-if="disableModal.open"
        class="fixed inset-0 bg-black/60 z-[9996] flex items-center justify-center"
        @mousedown.self="disableModal.open = false"
      >
        <div class="bg-gray-800 border border-gray-600 rounded-xl shadow-2xl p-4 min-w-[300px] max-w-sm">
          <div class="flex items-center gap-2 mb-3">
            <i class="fa-solid fa-ban text-gray-400"></i>
            <span class="text-sm font-semibold text-gray-200">停用 {{ disableModal.room?.name }}</span>
          </div>
          <p class="text-xs text-gray-400 mb-3">
            此房間目前有
            <span class="text-amber-300 font-semibold">{{ disableModal.affectedShifts.length }}</span>
            個科別時段，停用後要如何處理？
          </p>
          <div class="space-y-1 mb-4 max-h-32 overflow-y-auto">
            <div
              v-for="s in disableModal.affectedShifts" :key="s.id"
              class="flex items-center gap-2 px-2 py-1 rounded text-[10px]"
              :class="deptColor(s.dept)"
            >
              <span class="font-medium">{{ s.dept }}</span>
              <span class="opacity-70 font-mono ml-auto">{{ fmtTs(s.start_time) }}–{{ fmtTs(s.end_time) }}</span>
            </div>
          </div>
          <div class="flex flex-col gap-2">
            <button
              class="flex items-center gap-2 w-full py-2 px-3 text-xs rounded bg-gray-700 hover:bg-gray-600 text-gray-200 transition-colors text-left"
              @click="confirmDisableKeep"
            >
              <i class="fa-solid fa-clock-rotate-left text-[10px] text-gray-400"></i>
              保留時段（啟用後恢復）
            </button>
            <button
              class="flex items-center gap-2 w-full py-2 px-3 text-xs rounded bg-amber-800 hover:bg-amber-700 text-amber-200 transition-colors text-left"
              @click="confirmDisableStage"
            >
              <i class="fa-solid fa-box-archive text-[10px]"></i>
              移至暫存區
            </button>
            <button
              class="w-full py-2 px-3 text-xs rounded bg-transparent hover:bg-gray-700 text-gray-500 transition-colors"
              @click="disableModal.open = false"
            >
              取消
            </button>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- Conflict Resolution Modal -->
    <Teleport to="body">
      <div
        v-if="conflictModal.open"
        class="fixed inset-0 bg-black/60 z-[9996] flex items-center justify-center"
        @mousedown.self="conflictModal.open = false"
      >
        <div class="bg-gray-800 border border-amber-700/50 rounded-xl shadow-2xl p-4 min-w-[280px] max-w-sm">
          <div class="flex items-center gap-2 mb-3">
            <i class="fa-solid fa-triangle-exclamation text-amber-400"></i>
            <span class="text-sm font-semibold text-gray-200">時段衝突</span>
          </div>
          <div class="text-xs text-gray-400 space-y-1 mb-4">
            <div>
              拖入：<span class="text-gray-200 font-medium">{{ conflictModal.draggingShift?.dept }}</span>
              <span class="text-gray-500 ml-1 font-mono">{{ conflictModal.draggingShift ? fmtTs(conflictModal.draggingShift.start_time) + '–' + fmtTs(conflictModal.draggingShift.end_time) : '' }}</span>
            </div>
            <div>
              衝突：<span class="text-amber-300 font-medium">{{ conflictModal.conflictShift?.dept }}</span>
              <span class="text-gray-500 ml-1 font-mono">{{ conflictModal.conflictShift ? fmtTs(conflictModal.conflictShift.start_time) + '–' + fmtTs(conflictModal.conflictShift.end_time) : '' }}</span>
              <span class="text-gray-600 ml-1">@ {{ conflictModal.targetRoom }}</span>
            </div>
          </div>
          <div class="flex flex-col gap-2">
            <button
              v-if="!conflictModal.fromStaging"
              class="flex items-center gap-2 w-full py-2 px-3 text-xs rounded bg-blue-800 hover:bg-blue-700 text-blue-200 font-medium transition-colors"
              @click="confirmSwap"
            >
              <i class="fa-solid fa-arrows-rotate text-[10px]"></i>
              交換房間（{{ conflictModal.conflictShift?.dept }} → {{ conflictModal.sourceRoom }}）
            </button>
            <button
              class="flex items-center gap-2 w-full py-2 px-3 text-xs rounded bg-amber-800 hover:bg-amber-700 text-amber-200 font-medium transition-colors"
              @click="stageConflict"
            >
              <i class="fa-solid fa-box-archive text-[10px]"></i>
              移至暫存區（{{ conflictModal.conflictShift?.dept }} → 暫存）
            </button>
            <button
              class="w-full py-2 px-3 text-xs rounded bg-gray-700 hover:bg-gray-600 text-gray-400 transition-colors"
              @click="conflictModal.open = false"
            >
              取消
            </button>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- Task Context Menu -->
    <ContextMenu
      :show="taskMenu.open"
      :x="taskMenu.x"
      :y="taskMenu.y"
      @close="taskMenu.open = false"
    >
      <div class="px-3 py-1.5 text-[10px] text-gray-500 font-semibold border-b border-gray-700">
        病患：<span class="text-gray-200">{{ taskMenu.patientName }}</span>
      </div>
      <button
        class="flex items-center gap-2 w-full px-3 py-2 text-xs text-amber-300 hover:bg-amber-900/30 transition-colors text-left"
        @click="setTaskStatus('called')"
      >
        <i class="fa-solid fa-bell text-[10px]"></i>已叫刀
      </button>
      <button
        class="flex items-center gap-2 w-full px-3 py-2 text-xs text-green-300 hover:bg-green-900/30 transition-colors text-left"
        @click="setTaskStatus('in_surgery')"
      >
        <i class="fa-solid fa-scalpel text-[10px]"></i>手術中（進線）
      </button>
      <button
        class="flex items-center gap-2 w-full px-3 py-2 text-xs text-purple-300 hover:bg-purple-900/30 transition-colors text-left"
        @click="setTaskStatus('recovery')"
      >
        <i class="fa-solid fa-bed-pulse text-[10px]"></i>恢復室（結束）
      </button>
      <div class="border-t border-gray-700/60 my-0.5" />
      <div class="border-t border-gray-700/60 my-0.5" />
      <button
        class="flex items-center gap-2 w-full px-3 py-2 text-xs text-yellow-300 hover:bg-yellow-900/30 transition-colors text-left"
        @click="openSelfPay"
      >
        <i class="fa-solid fa-receipt text-[10px]"></i>添加自費備注
      </button>
      <button
        class="flex items-center gap-2 w-full px-3 py-2 text-xs text-red-400 hover:bg-red-900/30 transition-colors text-left"
        @click="unscheduleTask"
      >
        <i class="fa-solid fa-arrow-rotate-left text-[10px]"></i>移回待排定區
      </button>
    </ContextMenu>

    <!-- Patient Detail Modal -->
    <PatientDetailModal
      v-if="detailTask"
      :task="detailTask"
      @close="detailTaskId = null"
      @deleted="detailTaskId = null"
    />

    <!-- Self-Pay Picker Modal -->
    <SelfPayPickerModal
      v-if="selfPayTask"
      :task="selfPayTask"
      @close="selfPayTaskId = null"
    />

    <!-- Staff Assignment Context Menu -->
    <ContextMenu
      :show="staffMenu.open"
      :x="staffMenu.x"
      :y="staffMenu.y"
      @close="staffMenu.open = false"
    >
      <div class="px-3 py-1.5 text-[10px] text-gray-500 font-semibold border-b border-gray-700">
        人員：<span class="text-gray-200">{{ staffMenu.staffName }}</span>
      </div>
      <button
        class="flex items-center gap-2 w-full px-3 py-2 text-xs text-red-400 hover:bg-red-900/30 transition-colors text-left"
        @click="unassignStaff"
      >
        <i class="fa-solid fa-user-minus text-[10px]"></i>
        取消指派
      </button>
    </ContextMenu>

    <!-- Shift Block Context Menu -->
    <ContextMenu
      :show="shiftMenu.open"
      :x="shiftMenu.x"
      :y="shiftMenu.y"
      :width="200"
      @close="shiftMenu.open = false"
    >
      <div class="px-3 py-1.5 text-[10px] text-gray-500 font-semibold border-b border-gray-700">
        移動
        <span class="px-1 rounded ml-1" :class="shiftMenu.shift ? deptColor(shiftMenu.shift.dept) : ''">
          {{ shiftMenu.shift?.dept }}
        </span>
        <span class="text-gray-600 font-mono ml-1">{{ shiftMenu.shift ? fmtTs(shiftMenu.shift.start_time) + '–' + fmtTs(shiftMenu.shift.end_time) : '' }}</span>
      </div>
      <button
        v-for="r in rooms" :key="r.id"
        class="flex items-center gap-2 w-full px-3 py-1.5 text-xs transition-colors text-left"
        :class="shiftMenu.shift?.room_name === r.name
          ? 'text-gray-600 cursor-default'
          : 'text-gray-300 hover:bg-gray-700'"
        :disabled="shiftMenu.shift?.room_name === r.name"
        @click="assignShiftViaMenu(r.name)"
      >
        <i class="fa-solid fa-door-open text-[9px]" :class="shiftMenu.shift?.room_name === r.name ? 'text-gray-700' : 'text-gray-500'"></i>
        {{ r.name }}
        <span v-if="shiftMenu.shift?.room_name === r.name" class="ml-auto text-[9px] text-gray-600">目前</span>
        <span v-if="r.is_backup" class="ml-auto text-[9px] text-orange-500">備用</span>
      </button>
      <div class="border-t border-gray-700/60 my-0.5" />
      <button
        class="flex items-center gap-2 w-full px-3 py-1.5 text-xs text-amber-400 hover:bg-amber-900/30 transition-colors text-left"
        @click="stageShiftViaMenu"
      >
        <i class="fa-solid fa-box-archive text-[9px]"></i>移至暫存區
      </button>
    </ContextMenu>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, reactive } from "vue";
import type { Room, RoomScheduleEntry, StaffAssignment, SurgeryTask } from "../types";
import { useRoomsStore } from "../stores/rooms";
import { useTasksStore } from "../stores/tasks";
import { useAssignmentsStore } from "../stores/assignments";
import { useRoomShiftsDb, useDeptRulesDb } from "../composables/useDatabase";
import { useDragState } from "../composables/useDragState";
import PatientCard from "./PatientCard.vue";
import ContextMenu from "./ContextMenu.vue";
import PatientDetailModal from "./PatientDetailModal.vue";
import SelfPayPickerModal from "./SelfPayPickerModal.vue";

const detailTaskId = ref<number | null>(null);
const selfPayTaskId = ref<number | null>(null);

// ── 時段編輯模式 ──────────────────────────────────────────────────────────────────
const editMode      = ref(false);
const preEditShifts = ref<RoomScheduleEntry[]>([]);
const selectedShift = ref<RoomScheduleEntry | null>(null);

function enterEditMode() {
  preEditShifts.value  = shifts.value.map(s => ({ ...s }));
  stagedShifts.value   = [];
  selectedShift.value  = null;
  editMode.value       = true;
}

function cancelEditMode() {
  shifts.value         = preEditShifts.value;
  stagedShifts.value   = [];
  selectedShift.value  = null;
  editMode.value       = false;
  isDragging.value     = false;
  dragType.value       = null;
  draggingShift.value  = null;
}

async function saveEditMode() {
  await saveShifts();
  selectedShift.value = null;
  editMode.value      = false;
}

function onShiftBlockClick(shift: RoomScheduleEntry) {
  shiftMenu.open      = false;
  selectedShift.value = selectedShift.value?.id === shift.id ? null : { ...shift };
}

function onRoomColumnClick(roomName: string) {
  const sel = selectedShift.value;
  if (!sel) return;
  selectedShift.value = null;
  if (sel.room_name === roomName) return;
  handleShiftMove(sel, roomName, false);
}

// ── 科別色塊右鍵選單 ────────────────────────────────────────────────────────────
const shiftMenu = reactive({ open: false, x: 0, y: 0, shift: null as RoomScheduleEntry | null });

function onShiftRightClick(e: MouseEvent, shift: RoomScheduleEntry) {
  selectedShift.value = null;
  shiftMenu.open  = true;
  shiftMenu.x     = e.clientX;
  shiftMenu.y     = e.clientY;
  shiftMenu.shift = { ...shift };
}

function assignShiftViaMenu(roomName: string) {
  const shift = shiftMenu.shift;
  shiftMenu.open = false;
  if (!shift || shift.room_name === roomName) return;
  handleShiftMove(shift, roomName, false);
}

function stageShiftViaMenu() {
  const shift = shiftMenu.shift;
  shiftMenu.open = false;
  if (!shift) return;
  const idx = shifts.value.findIndex(s => s.id === shift.id);
  if (idx !== -1) {
    stagedShifts.value = [...stagedShifts.value, { ...shifts.value[idx] }];
    shifts.value       = shifts.value.filter((_, i) => i !== idx);
    saveShifts();
  }
}

const emit = defineEmits<{ "date-changed": [date: string] }>();

// ── 常數 ──────────────────────────────────────────────────────────────────────
const PX_PER_HOUR = 60;
const START_HOUR  = 7.5;
const TOTAL_HOURS = 24;
const TOTAL_PX    = TOTAL_HOURS * PX_PER_HOUR;
const HEADER_H    = 92;

const HOUR_LABELS = Array.from({ length: TOTAL_HOURS + 1 }, (_, i) => {
  const totalMins = Math.round(START_HOUR * 60) + i * 60;
  const h = Math.floor(totalMins / 60) % 24;
  const m = totalMins % 60;
  return {
    label: `${String(h).padStart(2, "0")}:${String(m).padStart(2, "0")}`,
    y: i * PX_PER_HOUR,
  };
});

const ROLES = [
  { value: "Scrub", label: "刷手",   activeClass: "bg-blue-700 border-blue-500 text-white",    inactiveClass: "border-blue-800 text-blue-400 hover:bg-blue-900/30" },
  { value: "Circ",  label: "流動",   activeClass: "bg-green-700 border-green-500 text-white",   inactiveClass: "border-green-800 text-green-400 hover:bg-green-900/30" },
  { value: "SA",    label: "助手",   activeClass: "bg-purple-700 border-purple-500 text-white",  inactiveClass: "border-purple-800 text-purple-400 hover:bg-purple-900/30" },
  { value: "R",     label: "住院師", activeClass: "bg-gray-600 border-gray-400 text-white",     inactiveClass: "border-gray-600 text-gray-400 hover:bg-gray-700/40" },
  { value: "VS",    label: "主治師", activeClass: "bg-red-700 border-red-500 text-white",       inactiveClass: "border-red-800 text-red-400 hover:bg-red-900/30" },
] as const;

// ── 科別配色 ──────────────────────────────────────────────────────────────────
const deptColorMap = ref<Record<string, string>>({});
async function loadDeptColors() {
  const rules = await useDeptRulesDb().getAll();
  const map: Record<string, string> = {};
  rules.forEach(r => map[r.dept] = r.color);
  deptColorMap.value = map;
}

function deptColor(dept: string): string {
  return deptColorMap.value[dept] || "bg-gray-800 text-gray-400";
}

// ── 角色徽章 ──────────────────────────────────────────────────────────────────
const ROLE_SHORT: Record<string, string> = {
  SA: "專", Scrub: "刷", Circ: "流", R: "師", VS: "主",
};
function roleShort(role: string): string {
  return ROLE_SHORT[role] ?? role.charAt(0);
}
const ROLE_BADGE: Record<string, string> = {
  SA:    "bg-purple-900/70 text-purple-300",
  Scrub: "bg-blue-900/70 text-blue-300",
  Circ:  "bg-green-900/70 text-green-300",
  R:     "bg-gray-700 text-gray-300",
  VS:    "bg-red-900/70 text-red-300",
};
function roleBadgeClass(role: string): string {
  return ROLE_BADGE[role] ?? "bg-gray-700 text-gray-400";
}

// ── 資料 ──────────────────────────────────────────────────────────────────────
const roomsStore       = useRoomsStore();
const tasksStore       = useTasksStore();
const assignmentsStore = useAssignmentsStore();

const detailTask = computed(() =>
  detailTaskId.value != null ? tasksStore.tasks.find(t => t.id === detailTaskId.value) ?? null : null
);
const selfPayTask = computed(() =>
  selfPayTaskId.value != null ? tasksStore.tasks.find(t => t.id === selfPayTaskId.value) ?? null : null
);
const roomShiftsDb   = useRoomShiftsDb();
const { isDragging, dragType } = useDragState();

const rooms       = computed<Room[]>(() => roomsStore.rooms);
const shifts      = ref<RoomScheduleEntry[]>([]);

const todayStr = ref("");

function getTodayStr(): string {
  const d = new Date();
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
}

function updateDate() {
  todayStr.value = getTodayStr();
}

const todayWeekday = computed(() => {
  if (!todayStr.value) return "";
  const [y, m, d] = todayStr.value.split("-").map(Number);
  return ["日","一","二","三","四","五","六"][new Date(y, m - 1, d).getDay()];
});

const isToday = computed(() => todayStr.value === getTodayStr());

function jumpToToday() {
  todayStr.value = getTodayStr();
  reload();
}

async function onDateChange() {
  emit("date-changed", todayStr.value);
  await reload();
  if (isToday.value) scrollToNow();
}

function roomSummary(roomName: string): { scheduled: number; active: number; recovery: number } {
  const tasks = scheduledTasksForRoom(roomName);
  return {
    scheduled: tasks.filter(t => t.status === "scheduled").length,
    active:    tasks.filter(t => t.status === "in_surgery" || t.status === "called").length,
    recovery:  tasks.filter(t => t.status === "recovery").length,
  };
}

const dayStart = computed(() => {
  if (!todayStr.value) return 0;
  const parts = todayStr.value.split('-').map(Number);
  const midnight = new Date(parts[0], parts[1]-1, parts[2]);
  return Math.floor(midnight.getTime() / 1000) + Math.round(START_HOUR * 3600);
});

function shiftsForRoom(roomName: string): RoomScheduleEntry[] {
  return shifts.value.filter((s) => s.room_name === roomName);
}
function assignmentsForRoom(roomName: string): StaffAssignment[] {
  return assignmentsStore.assignments.filter((a) => a.room_name === roomName);
}
function scheduledTasksForRoom(roomName: string): SurgeryTask[] {
  if (isToday.value) {
    // Today: show all non-waiting tasks assigned to this room
    return tasksStore.tasks.filter(t =>
      t.expected_room === roomName && t.status !== 'waiting'
    );
  }
  // Historical date: only show tasks whose scheduled_at falls within this day's window
  const dayEnd = dayStart.value + TOTAL_HOURS * 3600;
  return tasksStore.tasks.filter(t =>
    t.expected_room === roomName &&
    t.status !== 'waiting' &&
    t.scheduled_at !== null &&
    t.scheduled_at >= dayStart.value &&
    t.scheduled_at < dayEnd
  );
}

function timeToY(unixTs: number): number {
  let offset = unixTs - dayStart.value;
  // If it's the next day but within the 24h window
  if (offset < 0) offset += 86400;
  return (offset / 3600) * PX_PER_HOUR;
}

function blockStyle(shift: RoomScheduleEntry) {
  const top    = timeToY(shift.start_time);
  const height = Math.max(((shift.end_time - shift.start_time) / 3600) * PX_PER_HOUR, 16);
  return { top: `${top}px`, height: `${height}px` };
}

function taskBlockStyle(task: SurgeryTask) {
  const startTs = task.scheduled_at || task.created_at;
  const top = timeToY(startTs);
  const height = (task.est_time_mins / 60) * PX_PER_HOUR;
  return { top: `${top}px`, height: `${height}px` };
}

function fmtTs(ts: number): string {
  const d = new Date(ts * 1000);
  return `${String(d.getHours()).padStart(2, "0")}:${String(d.getMinutes()).padStart(2, "0")}`;
}

// ── 現在線 ────────────────────────────────────────────────────────────────────
const nowY = ref<number | null>(null);
let nowTimer: ReturnType<typeof setInterval> | null = null;

function updateNow() {
  const offset = Math.floor(Date.now() / 1000) - dayStart.value;
  nowY.value = (offset >= 0 && offset < TOTAL_HOURS * 3600)
    ? (offset / 3600) * PX_PER_HOUR
    : null;
}

const scrollEl = ref<HTMLElement | null>(null);
function scrollToNow() {
  if (nowY.value !== null && scrollEl.value) {
    scrollEl.value.scrollTop = Math.max(0, nowY.value - 120);
  }
}

async function reload() {
  editMode.value      = false;
  selectedShift.value = null;
  isDragging.value    = false;
  dragType.value      = null;
  draggingShift.value = null;
  stagedShifts.value  = [];
  [shifts.value] = await Promise.all([
    roomShiftsDb.getByDate(todayStr.value),
    assignmentsStore.load(todayStr.value),
    loadDeptColors(),
  ]);
  updateNow();
}

// ── 房間啟用切換 ─────────────────────────────────────────────────────────────────
const disableModal = reactive<{
  open: boolean;
  room: Room | null;
  affectedShifts: RoomScheduleEntry[];
}>({ open: false, room: null, affectedShifts: [] });

async function toggleRoomActive(room: Room) {
  if (!room.is_active) {
    // 重新啟用
    try {
      await roomsStore.edit({ ...room, is_active: true });
    } catch (e) {
      console.error("[Timeline] 啟用房間失敗:", e);
    }
    return;
  }
  // 停用前檢查當日是否有科別時段
  const affected = shiftsForRoom(room.name);
  if (affected.length > 0) {
    disableModal.room          = room;
    disableModal.affectedShifts = [...affected];
    disableModal.open          = true;
    return;
  }
  await applyDisableRoom(room, false);
}

async function applyDisableRoom(room: Room, stageShifts: boolean) {
  try {
    if (stageShifts) {
      stagedShifts.value = [...stagedShifts.value, ...disableModal.affectedShifts];
      shifts.value       = shifts.value.filter(s => s.room_name !== room.name);
      await saveShifts();
    }
    await roomsStore.edit({ ...room, is_active: false });
    const toUnschedule = tasksStore.tasks.filter(
      t => t.expected_room === room.name && t.status === 'scheduled'
    );
    for (const task of toUnschedule) {
      await tasksStore.edit({ ...task, status: 'waiting', expected_room: '', scheduled_at: null });
    }
  } catch (e) {
    console.error("[Timeline] 切換房間狀態失敗:", e);
  } finally {
    disableModal.open = false;
    disableModal.room = null;
    disableModal.affectedShifts = [];
  }
}

async function confirmDisableKeep() {
  if (!disableModal.room) return;
  await applyDisableRoom(disableModal.room, false);
}

async function confirmDisableStage() {
  if (!disableModal.room) return;
  await applyDisableRoom(disableModal.room, true);
}

// ── 拖放 ─────────────────────────────────────────────────────────────────────
const dragOverRoom = ref<string | null>(null);

interface DropState {
  staffName: string;
  roomName:  string;
  role:      string;
}
const dropState = ref<DropState | null>(null);

// ── 科別時段拖曳 ────────────────────────────────────────────────────────────────
const draggingShift   = ref<RoomScheduleEntry | null>(null);
const stagedShifts    = ref<RoomScheduleEntry[]>([]);
const stagingDragOver = ref(false);

interface ConflictModalState {
  open: boolean;
  draggingShift: RoomScheduleEntry | null;
  conflictShift: RoomScheduleEntry | null;
  targetRoom: string;
  sourceRoom: string | null;
  fromStaging: boolean;
}
const conflictModal = reactive<ConflictModalState>({
  open: false, draggingShift: null, conflictShift: null,
  targetRoom: "", sourceRoom: null, fromStaging: false,
});

function onShiftDragEnd() {
  isDragging.value    = false;
  dragType.value      = null;
  draggingShift.value = null;
}
function onStagedShiftDragStart(e: DragEvent, shift: RoomScheduleEntry) {
  isDragging.value    = true;
  dragType.value      = "shift";
  draggingShift.value = shift;
  e.dataTransfer!.effectAllowed = "move";
  e.dataTransfer!.setData("text/plain", JSON.stringify({ type: "shift", id: shift.id, fromStaging: true }));
}

async function saveShifts() {
  try {
    await roomShiftsDb.replaceByDate(todayStr.value, shifts.value);
  } catch (e) {
    console.error("[Timeline] 儲存科別時段失敗:", e);
  }
}

function applyShiftMove(shift: RoomScheduleEntry, targetRoom: string, fromStaging: boolean) {
  if (fromStaging) {
    stagedShifts.value = stagedShifts.value.filter(s => s.id !== shift.id);
    shifts.value = [...shifts.value, { ...shift, room_name: targetRoom }];
  } else {
    shifts.value = shifts.value.map(s => s.id === shift.id ? { ...s, room_name: targetRoom } : s);
  }
}

function handleShiftMove(shift: RoomScheduleEntry, targetRoom: string, fromStaging: boolean) {
  if (!fromStaging && shift.room_name === targetRoom) return;
  const conflicts = shifts.value.filter(s =>
    s.room_name === targetRoom &&
    s.id !== shift.id &&
    s.start_time < shift.end_time &&
    s.end_time > shift.start_time
  );
  if (conflicts.length === 0) {
    applyShiftMove(shift, targetRoom, fromStaging);
    saveShifts();
  } else {
    Object.assign(conflictModal, {
      open: true,
      draggingShift: { ...shift },
      conflictShift: { ...conflicts[0] },
      targetRoom,
      sourceRoom: fromStaging ? null : shift.room_name,
      fromStaging,
    });
  }
}

function confirmSwap() {
  const { draggingShift: ds, conflictShift: cs, targetRoom, sourceRoom } = conflictModal;
  if (!ds || !cs || !sourceRoom) return;
  shifts.value = shifts.value.map(s => {
    if (s.id === ds.id) return { ...s, room_name: targetRoom };
    if (s.id === cs.id) return { ...s, room_name: sourceRoom };
    return s;
  });
  saveShifts();
  conflictModal.open = false;
}

function stageConflict() {
  const { draggingShift: ds, conflictShift: cs, targetRoom, fromStaging } = conflictModal;
  if (!ds || !cs) return;
  const csEntry = shifts.value.find(s => s.id === cs.id);
  if (csEntry) {
    stagedShifts.value = [...stagedShifts.value, { ...csEntry }];
    shifts.value = shifts.value.filter(s => s.id !== cs.id);
  }
  applyShiftMove(ds, targetRoom, fromStaging);
  saveShifts();
  conflictModal.open = false;
}

function onDropToStaging(e: DragEvent) {
  stagingDragOver.value = false;
  const raw = e.dataTransfer?.getData("text/plain");
  if (!raw) return;
  try {
    const data = JSON.parse(raw) as { type: string; id?: number; fromStaging?: boolean };
    if (data.type === "shift" && data.id != null && !data.fromStaging) {
      const idx = shifts.value.findIndex(s => s.id === data.id);
      if (idx !== -1) {
        stagedShifts.value = [...stagedShifts.value, { ...shifts.value[idx] }];
        shifts.value = shifts.value.filter((_, i) => i !== idx);
        saveShifts();
      }
    }
  } catch {
    console.error("[Timeline] staging drop 解析失敗");
  }
}

function removeStaged(id: number) {
  stagedShifts.value = stagedShifts.value.filter(s => s.id !== id);
}

function onDragLeave(roomName: string) {
  if (dragOverRoom.value === roomName) dragOverRoom.value = null;
}

function onDrop(e: DragEvent, roomName: string) {
  dragOverRoom.value = null;
  const raw = e.dataTransfer?.getData("text/plain");
  if (!raw) return;
  try {
    const data = JSON.parse(raw) as { type: string; name?: string; id?: number; fromStaging?: boolean };
    if (data.type === "staff" && data.name) {
      dropState.value = { staffName: data.name, roomName, role: "Circ" };
    } else if (data.type === "task" && data.id != null) {
      assignTaskToRoom(data.id, roomName, e.clientY);
    } else if (data.type === "shift" && data.id != null) {
      const fromStaging = !!data.fromStaging;
      const shift = fromStaging
        ? stagedShifts.value.find(s => s.id === data.id)
        : shifts.value.find(s => s.id === data.id);
      if (shift) handleShiftMove(shift, roomName, fromStaging);
    }
  } catch {
    console.error("[Timeline] drop 資料解析失敗");
  }
}

function dropYToTime(clientY: number): number {
  const container = scrollEl.value;
  if (!container) return Math.floor(Date.now() / 1000);
  const zoom = parseFloat(getComputedStyle(document.documentElement).getPropertyValue('--app-zoom')) || 1;
  const rect = container.getBoundingClientRect();
  const yInContent = (clientY / zoom - rect.top) + container.scrollTop - HEADER_H;
  const hoursOffset = Math.max(0, yInContent) / PX_PER_HOUR;
  return dayStart.value + Math.round(hoursOffset * 3600);
}

async function assignTaskToRoom(taskId: number, roomName: string, clientY?: number) {
  const task = tasksStore.tasks.find(t => t.id === taskId);
  if (!task) return;
  try {
    const scheduled_at = clientY != null ? dropYToTime(clientY) : Math.floor(Date.now() / 1000);
    await tasksStore.edit({ ...task, expected_room: roomName, status: 'scheduled', scheduled_at });
  } catch (e) {
    console.error("[Timeline] 指派病患失敗:", e);
  }
}

async function selectRoleAndConfirm(role: string) {
  if (!dropState.value) return;
  dropState.value.role = role;
  await confirmDrop();
}

async function confirmDrop() {
  if (!dropState.value) return;
  const entry: StaffAssignment = {
    id: 0,
    staff_name: dropState.value.staffName,
    room_name:  dropState.value.roomName,
    date:       todayStr.value,
    role:       dropState.value.role,
  };
  try {
    await assignmentsStore.add(entry);
  } catch (e) {
    console.error("[Timeline] 儲存指派失敗:", e);
  }
  dropState.value = null;
}

// ── 人員退回與右鍵 ───────────────────────────────────────────────────────────────
const staffMenu = reactive({ open: false, x: 0, y: 0, id: 0, staffName: "" });
function onAssignmentRightClick(e: MouseEvent, a: StaffAssignment) {
  staffMenu.open = true;
  staffMenu.x = e.clientX;
  staffMenu.y = e.clientY;
  staffMenu.id = a.id;
  staffMenu.staffName = a.staff_name;
}

async function unassignStaff() {
  try {
    await assignmentsStore.remove(staffMenu.id);
  } catch (e) {
    console.error("[Timeline] 取消指派人員失敗:", e);
  }
  staffMenu.open = false;
}

function onAssignmentDragStart(e: DragEvent, a: StaffAssignment) {
  isDragging.value = true;
  dragType.value = 'assignment';
  e.dataTransfer!.effectAllowed = 'move';
  e.dataTransfer!.setData('text/plain', JSON.stringify({ type: 'assignment', id: a.id }));
}
function onAssignmentDragEnd() {
  isDragging.value = false;
  dragType.value = null;
}

// ── 病患退回右鍵 ───────────────────────────────────────────────────────────────
const taskMenu = reactive({ open: false, x: 0, y: 0, taskId: 0, patientName: "" });
function onTaskRightClick(e: MouseEvent, task: SurgeryTask) {
  taskMenu.open = true;
  taskMenu.x = e.clientX;
  taskMenu.y = e.clientY;
  taskMenu.taskId = task.id;
  taskMenu.patientName = task.patient_name;
}

function openSelfPay() {
  selfPayTaskId.value = taskMenu.taskId;
  taskMenu.open = false;
}

async function unscheduleTask() {
  const task = tasksStore.tasks.find(t => t.id === taskMenu.taskId);
  if (task) {
    try {
      await tasksStore.edit({ ...task, status: 'waiting', expected_room: '', scheduled_at: null });
    } catch (e) {
      console.error("[Timeline] 退回病患失敗:", e);
    }
  }
  taskMenu.open = false;
}

async function setTaskStatus(newStatus: string) {
  const task = tasksStore.tasks.find(t => t.id === taskMenu.taskId);
  if (task) {
    try {
      await tasksStore.edit({ ...task, status: newStatus });
    } catch (e) {
      console.error("[Timeline] 狀態更新失敗:", e);
    }
  }
  taskMenu.open = false;
}

// ── Lifecycle ─────────────────────────────────────────────────────────────────
onMounted(async () => {
  updateDate();
  emit("date-changed", todayStr.value);
  await roomsStore.load();
  await reload();
  updateNow();
  scrollToNow();
  nowTimer = setInterval(updateNow, 60000);
});

onUnmounted(() => {
  if (nowTimer) clearInterval(nowTimer);
});

defineExpose({ reload });
</script>
