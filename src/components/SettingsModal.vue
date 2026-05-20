<template>
  <div class="fixed inset-0 bg-black/70 z-50 flex items-center justify-center p-2" @mousedown.self="$emit('close')">
    <div class="bg-gray-800 rounded-xl shadow-2xl w-full max-w-3xl flex flex-col max-h-[96vh] overflow-hidden">

      <!-- Header -->
      <div class="flex items-center justify-between px-5 py-3 border-b border-gray-700 shrink-0">
        <h2 class="text-sm font-bold text-white"><i class="fa-solid fa-gear text-gray-400 mr-2"></i>設定</h2>
        <button class="text-gray-400 hover:text-white" @click="$emit('close')">
          <i class="fa-solid fa-xmark"></i>
        </button>
      </div>

      <!-- Tabs -->
      <div class="flex border-b border-gray-700 shrink-0 px-3 overflow-x-auto">
        <button
          v-for="t in TABS" :key="t.key"
          class="px-3 py-2 text-xs font-medium border-b-2 transition-colors whitespace-nowrap"
          :class="tab === t.key ? 'border-blue-500 text-blue-400' : 'border-transparent text-gray-400 hover:text-gray-200'"
          @click="tab = t.key"
        >
          <i :class="`fa-solid ${t.icon} mr-1`"></i>{{ t.label }}
        </button>
      </div>

      <!-- Body -->
      <div class="flex-1 overflow-y-auto px-4 py-3">

        <!-- ── 科別管理 ──────────────────────────────────────────── -->
        <template v-if="tab === 'depts'">
          <div class="flex items-center justify-between mb-4">
            <span class="text-sm text-gray-400">共 {{ deptRulesStore.rules.length }} 個科別設定</span>
            <button class="btn-primary text-xs" @click="startAddDept">
              <i class="fa-solid fa-plus mr-1"></i>新增科別設定
            </button>
          </div>

          <div class="space-y-2">
            <div
              v-for="rule in deptRulesStore.rules" :key="rule.dept"
              class="bg-gray-700/50 rounded-lg p-3 border border-gray-700"
            >
              <div class="flex items-center gap-3 mb-2">
                <div class="w-4 h-4 rounded border border-gray-500 shrink-0" :class="rule.color"></div>
                <span class="text-sm font-bold text-gray-100 flex-1">{{ rule.dept }}</span>
                <div class="flex gap-1.5">
                  <button class="text-gray-500 hover:text-blue-400 text-xs px-1" @click="startEditDept(rule)">
                    <i class="fa-solid fa-pen"></i>
                  </button>
                  <button class="text-gray-500 hover:text-red-400 text-xs px-1" @click="deptRulesStore.remove(rule.dept)">
                    <i class="fa-solid fa-trash"></i>
                  </button>
                </div>
              </div>
              <div class="flex items-center gap-4 text-[10px] text-gray-400 ml-7">
                <span><i class="fa-solid fa-arrow-up-wide-short mr-1"></i>權重加分: {{ rule.priority_bonus }}</span>
                <span class="truncate"><i class="fa-solid fa-thumbs-up mr-1"></i>偏好房間: {{ (rule.preferred_rooms || []).join(', ') || '無' }}</span>
              </div>
            </div>
            
            <div v-if="deptRulesStore.rules.length === 0" class="text-center text-gray-600 py-8 text-sm">
              尚未設定科別規則，建議設定科別專屬顏色
            </div>
          </div>

          <!-- Dept Form -->
          <div v-if="deptForm.open" class="mt-4 bg-gray-700/60 rounded-xl p-4 border border-gray-600">
            <div class="text-sm font-semibold text-gray-300 mb-3">設定科別：{{ deptForm.dept || '(新科別)' }}</div>
            <div class="grid grid-cols-2 gap-3 mb-3">
              <div>
                <label class="form-label">科別名稱 <span class="text-red-400">*</span></label>
                <input v-model="deptForm.dept" class="form-input" placeholder="骨科" :disabled="deptForm.isEdit" />
              </div>
              <div>
                <label class="form-label">優先權加分</label>
                <input v-model.number="deptForm.priority_bonus" type="number" class="form-input" />
              </div>
            </div>
            <div class="mb-3">
              <label class="form-label">科別代表色</label>
              <div class="flex flex-wrap gap-2 mt-1">
                <button
                  v-for="c in PRESET_COLORS" :key="c"
                  class="w-8 h-8 rounded-lg border-2 transition-all"
                  :class="[c, deptForm.color === c ? 'border-white scale-110 shadow-lg' : 'border-transparent opacity-60 hover:opacity-100']"
                  @click="deptForm.color = c"
                ></button>
              </div>
            </div>
            <div class="flex justify-end gap-2 mt-4">
              <button class="btn-secondary text-xs" @click="deptForm.open = false">取消</button>
              <button class="btn-primary text-xs" :disabled="!deptForm.dept.trim()" @click="saveDept">儲存設定</button>
            </div>
          </div>
        </template>

        <!-- ── 房間管理 ──────────────────────────────────────────── -->
        <template v-if="tab === 'rooms'">
          <div class="flex items-center justify-between mb-4">
            <span class="text-sm text-gray-400">共 {{ roomsStore.rooms.length }} 間手術室</span>
            <div class="flex gap-2">
              <button class="btn-secondary text-xs flex items-center gap-1.5" @click="triggerXlsx">
                <i class="fa-solid fa-file-excel text-green-400"></i>匯入刀房科別時段 (.xlsx)
              </button>
              <input ref="xlsxInput" type="file" accept=".xlsx,.xls" class="hidden" @change="onXlsxFile" />
              <button class="btn-secondary text-xs" @click="toggleBatchRoom">
                <i class="fa-solid fa-list-ul mr-1"></i>批量新增
              </button>
              <button class="btn-primary text-xs" @click="startAddRoom">
                <i class="fa-solid fa-plus mr-1"></i>新增房間
              </button>
            </div>
          </div>

          <!-- Batch room add panel -->
          <div v-if="batchRoom.open" class="mb-4 bg-gray-700/60 rounded-xl p-4 border border-blue-700/40">
            <div class="text-sm font-semibold text-gray-300 mb-1">批量新增房間</div>
            <p class="text-xs text-gray-500 mb-2">每行一個房間名稱，已存在的名稱自動略過</p>
            <textarea
              v-model="batchRoom.text"
              rows="6"
              class="form-input resize-none font-mono text-xs"
              placeholder="OR1&#10;OR2&#10;OR3&#10;急救刀房&#10;備用刀房"
            />
            <div class="flex items-center justify-between mt-3">
              <span class="text-xs text-gray-500">
                待新增 <span class="text-white font-medium">{{ batchRoomNew.length }}</span> 間
                <template v-if="batchRoomSkip.length">，略過重複 {{ batchRoomSkip.length }} 間</template>
              </span>
              <div class="flex gap-2">
                <button class="btn-secondary text-xs" @click="batchRoom.open = false">取消</button>
                <button
                  class="btn-primary text-xs"
                  :disabled="batchRoomNew.length === 0 || batchRoom.saving"
                  @click="saveBatchRooms"
                >
                  <i v-if="batchRoom.saving" class="fa-solid fa-spinner animate-spin mr-1"></i>
                  新增 {{ batchRoomNew.length }} 間
                </button>
              </div>
            </div>
          </div>

          <!-- Room list -->
          <div class="space-y-1.5">
            <div
              v-for="(room, idx) in roomsStore.rooms" :key="room.id"
              class="flex items-center gap-3 bg-gray-700/50 rounded-lg px-3 py-2.5"
            >
              <span class="text-gray-500 text-xs w-5 text-center">{{ idx + 1 }}</span>
              <i v-if="room.is_backup" class="fa-solid fa-circle-exclamation text-orange-400 text-xs" title="備用刀房"></i>
              <i v-else class="fa-solid fa-door-open text-blue-400 text-xs"></i>
              <span class="flex-1 text-sm text-gray-100 font-medium">{{ room.name }}</span>
              <span v-if="room.is_backup" class="text-[10px] text-orange-400 bg-orange-900/40 px-1.5 py-0.5 rounded">備用</span>
              <button class="text-gray-500 hover:text-blue-400 text-xs px-1" @click="startEditRoom(room)">
                <i class="fa-solid fa-pen"></i>
              </button>
              <button class="text-gray-500 hover:text-red-400 text-xs px-1" @click="deleteRoom(room.id)">
                <i class="fa-solid fa-trash"></i>
              </button>
            </div>
            <div v-if="roomsStore.rooms.length === 0" class="text-center text-gray-600 py-8 text-sm">
              尚未設定任何房間，請點擊「新增房間」或匯入月排班 Excel
            </div>
          </div>

          <!-- Room add/edit inline form -->
          <div v-if="roomForm.open" class="mt-4 bg-gray-700/60 rounded-xl p-4 border border-gray-600">
            <div class="text-sm font-semibold text-gray-300 mb-3">
              {{ roomForm.id ? '編輯房間' : '新增房間' }}
            </div>
            <div class="grid grid-cols-2 gap-3">
              <div>
                <label class="form-label">房間名稱 <span class="text-red-400">*</span></label>
                <input v-model="roomForm.name" class="form-input" placeholder="OR1" />
              </div>
              <div>
                <label class="form-label">顯示順序</label>
                <input v-model.number="roomForm.display_order" type="number" min="0" class="form-input" />
              </div>
            </div>
            <label class="flex items-center gap-2 mt-3 cursor-pointer">
              <input type="checkbox" v-model="roomForm.is_backup" class="rounded" />
              <span class="text-sm text-gray-300">備用刀房（二線機制才啟用）</span>
            </label>
            <div class="flex justify-end gap-2 mt-3">
              <button class="btn-secondary text-xs" @click="roomForm.open = false">取消</button>
              <button class="btn-primary text-xs" :disabled="!roomForm.name.trim()" @click="saveRoom">
                {{ roomForm.id ? '儲存' : '新增' }}
              </button>
            </div>
          </div>

          <!-- PDF 代碼對應 -->
          <div class="mt-6 border-t border-gray-700/60 pt-4">
            <div class="flex items-center justify-between mb-2">
              <div>
                <span class="text-sm font-semibold text-gray-300">HIS PDF 代碼對應</span>
                <p class="text-xs text-gray-500 mt-0.5">匯入 PDF 時，將房號代碼轉換為系統房間名稱（如 05 → OR5）</p>
              </div>
              <button class="btn-secondary text-xs" @click="addCodeMapRow">
                <i class="fa-solid fa-plus mr-1"></i>新增
              </button>
            </div>
            <div class="space-y-1.5">
              <div v-for="(entry, i) in codeMapRows" :key="i" class="flex items-center gap-2">
                <input v-model="entry.pdfCode" class="form-input flex-1 font-mono" placeholder="PDF 代碼（如 05）" />
                <i class="fa-solid fa-arrow-right text-gray-600 text-[10px] shrink-0"></i>
                <input v-model="entry.roomName" list="room-list-map" class="form-input flex-1" placeholder="系統房間（如 OR5）" />
                <button class="text-gray-600 hover:text-red-400 text-xs shrink-0" @click="removeCodeMapRow(i)">
                  <i class="fa-solid fa-xmark"></i>
                </button>
              </div>
              <div v-if="codeMapRows.length === 0" class="text-xs text-gray-600 py-2 pl-1">
                尚無對應規則，PDF 代碼將直接寫入
              </div>
            </div>
            <datalist id="room-list-map">
              <option v-for="r in roomsStore.rooms" :key="r.id" :value="r.name" />
            </datalist>
            <div class="flex items-center justify-end gap-2 mt-3">
              <span v-if="codeMapSaveMsg" class="text-xs" :class="codeMapSaveMsg.ok ? 'text-green-400' : 'text-red-400'">
                {{ codeMapSaveMsg.msg }}
              </span>
              <button class="btn-primary text-xs" @click="saveCodeMap">儲存對應規則</button>
            </div>
          </div>

          <!-- 房間群組（一對多展開） -->
          <div class="mt-4 border-t border-gray-700/60 pt-4">
            <div class="flex items-center justify-between mb-2">
              <div>
                <span class="text-sm font-semibold text-gray-300">房間群組</span>
                <p class="text-xs text-gray-500 mt-0.5">科別時段匯入時，群組名稱自動展開為多間實體房間</p>
              </div>
              <button class="btn-secondary text-xs" @click="addGroupRow">
                <i class="fa-solid fa-plus mr-1"></i>新增
              </button>
            </div>
            <div class="space-y-1.5">
              <div v-for="(g, i) in roomGroupRows" :key="i" class="flex items-center gap-2">
                <input v-model="g.name" class="form-input w-24 font-mono shrink-0" placeholder="群組名（如 局麻）" />
                <i class="fa-solid fa-arrow-right text-gray-600 text-[10px] shrink-0"></i>
                <input v-model="g.rooms" class="form-input flex-1 font-mono text-xs" placeholder="OR1, OR2, OR3（逗號分隔）" />
                <button class="text-gray-600 hover:text-red-400 text-xs shrink-0" @click="removeGroupRow(i)">
                  <i class="fa-solid fa-xmark"></i>
                </button>
              </div>
              <div v-if="roomGroupRows.length === 0" class="text-xs text-gray-600 py-2 pl-1">
                尚無群組，科別時段直接對應單一房間
              </div>
            </div>
            <div class="flex items-center justify-end gap-2 mt-3">
              <span v-if="roomGroupSaveMsg" class="text-xs" :class="roomGroupSaveMsg.ok ? 'text-green-400' : 'text-red-400'">
                {{ roomGroupSaveMsg.msg }}
              </span>
              <button class="btn-primary text-xs" @click="saveRoomGroupRows">儲存群組設定</button>
            </div>
          </div>

          <!-- XLSX preview -->
          <div v-if="xlsxPreview.length > 0 || xlsxErrors.length > 0" class="mt-4 bg-gray-900/60 rounded-xl p-4 border" :class="xlsxPreview.length > 0 ? 'border-green-700/50' : 'border-red-900/50'">
            <div class="flex items-center justify-between mb-3">
              <div class="text-sm font-semibold" :class="xlsxPreview.length > 0 ? 'text-green-300' : 'text-red-400'">
                <i class="fa-solid" :class="xlsxPreview.length > 0 ? 'fa-table' : 'fa-circle-exclamation'"></i>
                {{ xlsxPreview.length > 0 ? `預覽刀房時段匯入 — ${xlsxMonth}（共 ${xlsxPreview.length} 筆）` : '檔案解析失敗' }}
              </div>
              <div class="flex gap-2">
                <button class="btn-secondary text-xs" @click="xlsxPreview = []; xlsxErrors = []">關閉</button>
                <button v-if="xlsxPreview.length > 0" class="btn-primary text-xs" :disabled="importSaving" @click="confirmXlsxImport">
                  <i v-if="importSaving" class="fa-solid fa-spinner animate-spin mr-1"></i>
                  確認匯入
                </button>
              </div>
            </div>
            
            <div v-if="xlsxErrors.length" class="mb-2 p-2 bg-red-950/20 border border-red-900/30 rounded text-[10px] text-red-400 max-h-32 overflow-y-auto">
              <div class="font-bold mb-1">解析錯誤 ({{ xlsxErrors.length }} 筆):</div>
              <div v-for="(err, i) in xlsxErrors" :key="i" class="font-mono">{{ err }}</div>
            </div>

            <div v-if="xlsxPreview.length > 0" class="overflow-x-auto max-h-52">
              <table class="w-full text-xs text-gray-300 border-collapse">
                <thead>
                  <tr class="text-gray-500 border-b border-gray-700">
                    <th class="text-left py-1 px-2">日期</th>
                    <th class="text-left py-1 px-2">房間</th>
                    <th class="text-left py-1 px-2">科別</th>
                    <th class="text-left py-1 px-2">開始</th>
                    <th class="text-left py-1 px-2">結束</th>
                    <th class="text-left py-1 px-2">備注</th>
                  </tr>
                </thead>
                <tbody>
                  <tr v-for="(e, i) in xlsxPreview.slice(0, 10)" :key="i" class="border-b border-gray-800">
                    <td class="py-1 px-2">{{ e.date }}</td>
                    <td class="py-1 px-2">{{ e.room_name }}</td>
                    <td class="py-1 px-2">{{ e.dept }}</td>
                    <td class="py-1 px-2">{{ fmtTs(e.start_time) }}</td>
                    <td class="py-1 px-2">{{ fmtTs(e.end_time) }}</td>
                    <td class="py-1 px-2 text-gray-500">{{ e.notes }}</td>
                  </tr>
                </tbody>
              </table>
              <div v-if="xlsxPreview.length > 10" class="text-center text-gray-600 text-xs py-1">
                … 還有 {{ xlsxPreview.length - 10 }} 筆
              </div>
            </div>
          </div>
        </template>

        <!-- ── 人員管理 ──────────────────────────────────────────── -->
        <template v-if="tab === 'staff'">
          <div class="flex items-center justify-between mb-4">
            <span class="text-sm text-gray-400">共 {{ staffStore.staffList.length }} 名人員</span>
            <div class="flex gap-2">
              <div class="flex flex-col items-end">
                <button class="btn-secondary text-xs flex items-center gap-1.5" @click="triggerStaffXlsx">
                  <i class="fa-solid fa-calendar-check text-green-400"></i>匯入人員月排班 (.xlsx)
                </button>
                <span class="text-[9px] text-gray-500 mt-1">※ 用於人員指派及 Extra 線勞基法判別</span>
              </div>
              <input ref="staffXlsxInput" type="file" accept=".xlsx,.xls" class="hidden" @change="onStaffXlsxFile" />
              <button class="btn-secondary text-xs" @click="toggleBatchStaff">
                <i class="fa-solid fa-list-ul mr-1"></i>批量新增
              </button>
              <button class="btn-primary text-xs" @click="startAddStaff">
                <i class="fa-solid fa-plus mr-1"></i>新增人員
              </button>
            </div>
          </div>

          <!-- Staff assignment xlsx preview -->
          <div v-if="staffXlsxPreview.length > 0 || staffXlsxErrors.length > 0" class="mb-4 bg-gray-900/60 rounded-xl p-4 border" :class="staffXlsxPreview.length > 0 ? 'border-green-700/50' : 'border-red-900/50'">
            <div class="flex items-center justify-between mb-3">
              <div class="text-sm font-semibold" :class="staffXlsxPreview.length > 0 ? 'text-green-300' : 'text-red-400'">
                <i class="fa-solid" :class="staffXlsxPreview.length > 0 ? 'fa-calendar-days' : 'fa-circle-exclamation'"></i>
                {{ staffXlsxPreview.length > 0 ? `預覽人員月排班 — ${staffXlsxMonth}（共 ${staffXlsxPreview.length} 筆）` : '班表解析失敗' }}
              </div>
              <div class="flex gap-2">
                <button class="btn-secondary text-xs" @click="staffXlsxPreview = []; staffXlsxErrors = []">關閉</button>
                <button v-if="staffXlsxPreview.length > 0" class="btn-primary text-xs" :disabled="staffXlsxSaving" @click="confirmStaffXlsxImport">
                  <i v-if="staffXlsxSaving" class="fa-solid fa-spinner animate-spin mr-1"></i>
                  確認匯入
                </button>
              </div>
            </div>

            <div v-if="staffXlsxErrors.length" class="mb-2 p-2 bg-red-950/20 border border-red-900/30 rounded text-[10px] text-red-400 max-h-32 overflow-y-auto">
              <div class="font-bold mb-1">解析錯誤 ({{ staffXlsxErrors.length }} 筆):</div>
              <div v-for="(err, i) in staffXlsxErrors" :key="i" class="font-mono">{{ err }}</div>
            </div>

            <div v-if="staffXlsxPreview.length > 0" class="overflow-x-auto max-h-48">
              <table class="w-full text-xs text-gray-300 border-collapse">
                <thead>
                  <tr class="text-gray-500 border-b border-gray-700">
                    <th class="text-left py-1 px-2">日期</th>
                    <th class="text-left py-1 px-2">姓名</th>
                    <th class="text-left py-1 px-2">班別</th>
                  </tr>
                </thead>
                <tbody>
                  <tr v-for="(a, i) in staffXlsxPreview.slice(0, 10)" :key="i" class="border-b border-gray-800">
                    <td class="py-1 px-2">{{ a.date }}</td>
                    <td class="py-1 px-2">{{ a.staff_name }}</td>
                    <td class="py-1 px-2">{{ a.shift_name }}</td>
                  </tr>
                </tbody>
              </table>
              <div v-if="staffXlsxPreview.length > 10" class="text-center text-gray-600 text-xs py-1">
                … 還有 {{ staffXlsxPreview.length - 10 }} 筆
              </div>
            </div>
            <p class="text-[10px] text-gray-600 mt-2">
              匯入後將取代 {{ staffXlsxMonth }} 整月的人員班表資料
            </p>
          </div>

          <!-- Batch staff add panel -->
          <div v-if="batchStaff.open" class="mb-4 bg-gray-700/60 rounded-xl p-4 border border-blue-700/40">
            <div class="text-sm font-semibold text-gray-300 mb-3">批量新增人員</div>
            <div class="overflow-x-auto">
              <table class="w-full text-xs border-collapse">
                <thead>
                  <tr class="text-gray-500 border-b border-gray-600">
                    <th class="text-left pb-2 pr-2 font-medium w-32">姓名 <span class="text-red-400">*</span></th>
                    <th class="text-left pb-2 pr-2 font-medium w-40">類別</th>
                    <th class="text-left pb-2 pr-2 font-medium">所屬單位</th>
                    <th class="pb-2 w-6"></th>
                  </tr>
                </thead>
                <tbody>
                  <tr v-for="(row, i) in batchStaff.rows" :key="i" class="border-b border-gray-700/40">
                    <td class="py-1 pr-2">
                      <input
                        v-model="row.name"
                        class="w-full bg-gray-800 border border-gray-600 rounded px-2 py-1 text-gray-100 placeholder-gray-600 outline-none focus:border-blue-500"
                        placeholder="姓名"
                      />
                    </td>
                    <td class="py-1 pr-2">
                      <select v-model="row.staff_category" class="w-full bg-gray-800 border border-gray-600 rounded px-2 py-1 text-gray-100 outline-none focus:border-blue-500">
                        <option v-for="(label, key) in CATEGORY_LABELS" :key="key" :value="key">{{ label }}</option>
                      </select>
                    </td>
                    <td class="py-1 pr-2">
                      <input
                        v-if="row.staff_category === 'cross_train'"
                        v-model="row.unit"
                        class="w-full bg-gray-800 border border-gray-600 rounded px-2 py-1 text-gray-100 placeholder-gray-600 outline-none focus:border-blue-500"
                        placeholder="單位名稱"
                      />
                      <span v-else class="text-gray-700">—</span>
                    </td>
                    <td class="py-1 text-center">
                      <button class="text-gray-600 hover:text-red-400" @click="batchStaff.rows.splice(i, 1)">
                        <i class="fa-solid fa-xmark"></i>
                      </button>
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
            <button class="mt-2 text-xs text-blue-400 hover:text-blue-300 flex items-center gap-1" @click="addBatchStaffRow">
              <i class="fa-solid fa-plus"></i> 新增一行
            </button>
            <div class="flex items-center justify-between mt-3">
              <span class="text-xs text-gray-500">
                有效筆數：<span class="text-white font-medium">{{ validBatchStaffRows }}</span>
              </span>
              <div class="flex gap-2">
                <button class="btn-secondary text-xs" @click="batchStaff.open = false">取消</button>
                <button
                  class="btn-primary text-xs"
                  :disabled="validBatchStaffRows === 0 || batchStaff.saving"
                  @click="saveBatchStaff"
                >
                  <i v-if="batchStaff.saving" class="fa-solid fa-spinner animate-spin mr-1"></i>
                  儲存 {{ validBatchStaffRows }} 筆
                </button>
              </div>
            </div>
          </div>

          <!-- Staff add/edit inline form -->
          <div v-if="staffForm.open" class="mb-4 bg-gray-700/60 rounded-xl p-4 border border-gray-600">
            <div class="text-sm font-semibold text-gray-300 mb-3">
              {{ staffForm.id ? '編輯人員' : '新增人員' }}
            </div>
            <div class="grid grid-cols-2 gap-3">
              <div>
                <label class="form-label">姓名 <span class="text-red-400">*</span></label>
                <input v-model="staffForm.name" class="form-input" placeholder="王小明" />
              </div>
              <div>
                <label class="form-label">類別 <span class="text-red-400">*</span></label>
                <select v-model="staffForm.staff_category" class="form-input">
                  <option v-for="(label, key) in CATEGORY_LABELS" :key="key" :value="key">{{ label }}</option>
                </select>
              </div>
              <div v-if="staffForm.staff_category === 'cross_train'">
                <label class="form-label">所屬單位 <span class="text-red-400">*</span></label>
                <input v-model="staffForm.unit" class="form-input" placeholder="心臟外科加護病房" />
              </div>
            </div>
            <div class="flex gap-4 mt-3">
              <label class="flex items-center gap-2 cursor-pointer">
                <input type="checkbox" v-model="staffForm.is_on_call" class="rounded" />
                <span class="text-sm text-gray-300">今日值班 (二線待命)</span>
              </label>
              <label class="flex items-center gap-2 cursor-pointer">
                <input type="checkbox" v-model="staffForm.is_volunteer_extra" class="rounded" />
                <span class="text-sm text-gray-300">可加 Extra</span>
              </label>
            </div>
            <div class="flex justify-end gap-2 mt-3">
              <button class="btn-secondary text-xs" @click="staffForm.open = false">取消</button>
              <button
                class="btn-primary text-xs"
                :disabled="!staffForm.name.trim() || (staffForm.staff_category === 'cross_train' && !staffForm.unit.trim())"
                @click="saveStaff"
              >
                {{ staffForm.id ? '儲存' : '新增' }}
              </button>
            </div>
          </div>

          <!-- Staff list grouped by category -->
          <div v-for="cat in CATEGORIES" :key="cat">
            <div v-if="staffByCategory[cat]?.length">
              <div class="flex items-center gap-2 mb-2 mt-3">
                <span class="text-xs font-semibold uppercase tracking-widest" :class="categoryColor(cat)">
                  {{ CATEGORY_LABELS[cat] }}
                </span>
                <div class="flex-1 border-t border-gray-700/60"></div>
                <span class="text-[10px] text-gray-600">{{ staffByCategory[cat].length }} 人</span>
              </div>
              <div class="space-y-1">
                <div
                  v-for="s in staffByCategory[cat]" :key="s.id"
                  class="flex items-center gap-3 bg-gray-700/40 rounded-lg px-3 py-2"
                >
                  <i class="fa-solid fa-user text-gray-500 text-xs w-4 text-center"></i>
                  <span class="flex-1 text-sm text-gray-100">{{ s.name }}</span>
                  <span v-if="s.unit" class="text-[10px] text-gray-500 truncate max-w-[100px]">{{ s.unit }}</span>
                  <span v-if="s.is_on_call" class="text-[10px] text-yellow-400 bg-yellow-900/40 px-1.5 py-0.5 rounded">值班</span>
                  <span v-if="s.is_volunteer_extra" class="text-[10px] text-purple-400 bg-purple-900/40 px-1.5 py-0.5 rounded">Extra</span>
                  <button class="text-gray-500 hover:text-blue-400 text-xs px-1" @click="startEditStaff(s)">
                    <i class="fa-solid fa-pen"></i>
                  </button>
                  <button class="text-gray-500 hover:text-red-400 text-xs px-1" @click="deleteStaff(s.id)">
                    <i class="fa-solid fa-trash"></i>
                  </button>
                </div>
              </div>
            </div>
          </div>
          <div v-if="staffStore.staffList.length === 0" class="text-center text-gray-600 py-8 text-sm">
            尚未設定人員，請點擊「新增人員」
          </div>
        </template>

        <!-- ── 自費項目 ──────────────────────────────────────────── -->
        <template v-if="tab === 'selfpay'">
          <div class="flex items-center justify-between mb-4">
            <span class="text-sm text-gray-400">共 {{ selfPayItems.length }} 個自費項目</span>
            <div class="flex gap-2">
              <button class="btn-secondary text-xs flex items-center gap-1.5" @click="downloadSelfPayTemplate">
                <i class="fa-solid fa-file-arrow-down text-green-400"></i>下載範本
              </button>
              <button class="btn-secondary text-xs flex items-center gap-1.5" @click="selfPayImportInput?.click()">
                <i class="fa-solid fa-file-import text-blue-400"></i>匯入 CSV
              </button>
              <input ref="selfPayImportInput" type="file" accept=".csv,.txt" class="hidden" @change="onSelfPayCsvFile" />
              <button class="btn-primary text-xs" @click="startAddSelfPay">
                <i class="fa-solid fa-plus mr-1"></i>新增項目
              </button>
            </div>
          </div>

          <!-- Import preview -->
          <div v-if="selfPayImport.rows.length > 0" class="mb-4 bg-gray-900/60 rounded-xl p-4 border border-blue-700/40">
            <div class="flex items-center justify-between mb-2">
              <span class="text-sm font-semibold text-blue-300">
                <i class="fa-solid fa-table mr-1.5"></i>預覽匯入（{{ selfPayImport.rows.length }} 筆）
              </span>
              <div class="flex gap-2">
                <button class="btn-secondary text-xs" @click="selfPayImport.rows = []">取消</button>
                <button class="btn-primary text-xs" :disabled="selfPayImport.saving" @click="confirmSelfPayImport">
                  <i v-if="selfPayImport.saving" class="fa-solid fa-spinner animate-spin mr-1"></i>
                  確認匯入
                </button>
              </div>
            </div>
            <div v-if="selfPayImport.errors.length" class="mb-2 text-[10px] text-red-400 bg-red-950/30 rounded p-2">
              略過 {{ selfPayImport.errors.length }} 筆無效行
            </div>
            <div class="overflow-x-auto max-h-40">
              <table class="w-full text-xs text-gray-300 border-collapse">
                <thead><tr class="text-gray-500 border-b border-gray-700">
                  <th class="text-left py-1 px-2">項目名稱</th>
                  <th class="text-left py-1 px-2">費用</th>
                  <th class="text-left py-1 px-2">備注</th>
                </tr></thead>
                <tbody>
                  <tr v-for="(r, i) in selfPayImport.rows.slice(0, 8)" :key="i" class="border-b border-gray-800">
                    <td class="py-1 px-2">{{ r.name }}</td>
                    <td class="py-1 px-2 font-mono">${{ r.price }}</td>
                    <td class="py-1 px-2 text-gray-500">{{ r.notes }}</td>
                  </tr>
                </tbody>
              </table>
              <div v-if="selfPayImport.rows.length > 8" class="text-center text-gray-600 text-xs py-1">
                … 還有 {{ selfPayImport.rows.length - 8 }} 筆
              </div>
            </div>
          </div>

          <div class="space-y-1.5 mb-4">
            <div
              v-for="item in selfPayItems" :key="item.id"
              class="flex items-center gap-3 bg-gray-700/50 rounded-lg px-3 py-2.5"
            >
              <i class="fa-solid fa-receipt text-yellow-400 text-xs w-4 text-center"></i>
              <span class="flex-1 text-sm text-gray-100 font-medium">{{ item.name }}</span>
              <span v-if="item.price > 0" class="text-xs text-yellow-300 font-mono shrink-0">${{ item.price.toLocaleString() }}</span>
              <span v-if="item.notes" class="text-[10px] text-gray-500 truncate max-w-[100px]">{{ item.notes }}</span>
              <button class="text-gray-500 hover:text-blue-400 text-xs px-1" @click="startEditSelfPay(item)">
                <i class="fa-solid fa-pen"></i>
              </button>
              <button class="text-gray-500 hover:text-red-400 text-xs px-1" @click="deleteSelfPay(item.id)">
                <i class="fa-solid fa-trash"></i>
              </button>
            </div>
            <div v-if="selfPayItems.length === 0" class="text-center text-gray-600 py-8 text-sm">
              尚未設定自費項目，請點擊「新增項目」
            </div>
          </div>

          <!-- Self-Pay add/edit inline form -->
          <div v-if="selfPayForm.open" class="bg-gray-700/60 rounded-xl p-4 border border-gray-600">
            <div class="text-sm font-semibold text-gray-300 mb-3">
              {{ selfPayForm.id ? '編輯自費項目' : '新增自費項目' }}
            </div>
            <div class="grid grid-cols-2 gap-3">
              <div class="col-span-2">
                <label class="form-label">項目名稱 <span class="text-red-400">*</span></label>
                <input v-model="selfPayForm.name" class="form-input" placeholder="自費手套" />
              </div>
              <div>
                <label class="form-label">費用（元）</label>
                <input v-model.number="selfPayForm.price" type="number" min="0" class="form-input" placeholder="0" />
              </div>
              <div>
                <label class="form-label">備注</label>
                <input v-model="selfPayForm.notes" class="form-input" placeholder="選填說明" />
              </div>
            </div>
            <div class="flex justify-end gap-2 mt-3">
              <button class="btn-secondary text-xs" @click="selfPayForm.open = false">取消</button>
              <button class="btn-primary text-xs" :disabled="!selfPayForm.name.trim()" @click="saveSelfPay">
                {{ selfPayForm.id ? '儲存' : '新增' }}
              </button>
            </div>
          </div>
        </template>

        <!-- ── 顯示設定 ──────────────────────────────────────────── -->
        <template v-if="tab === 'display'">
          <div class="max-w-sm space-y-6">
            <div>
              <label class="form-label mb-3">介面縮放比例</label>
              <div class="flex items-center gap-4 mb-3">
                <span class="text-2xl font-bold font-mono text-blue-300 w-16 text-center">
                  {{ Math.round(zoomValue * 100) }}%
                </span>
                <div class="flex gap-1.5">
                  <button
                    v-for="preset in ZOOM_PRESETS" :key="preset"
                    class="px-2 py-1 rounded text-xs transition-colors"
                    :class="Math.round(zoomValue * 100) === Math.round(preset * 100)
                      ? 'bg-blue-600 text-white'
                      : 'bg-gray-700 text-gray-400 hover:bg-gray-600'"
                    @click="applyZoom(preset)"
                  >{{ Math.round(preset * 100) }}%</button>
                </div>
              </div>
              <input
                type="range"
                v-model.number="zoomValue"
                min="0.5" max="2.0" step="0.05"
                class="w-full accent-blue-500"
                @input="applyZoom(zoomValue)"
              />
              <div class="flex justify-between text-xs text-gray-600 mt-1">
                <span>50%</span><span>100%</span><span>150%</span><span>200%</span>
              </div>
              <p class="text-xs text-gray-600 mt-3">調整後立即生效，重新啟動自動套用上次設定</p>
            </div>
          </div>
        </template>

        <!-- ── 雲端設定 ──────────────────────────────────────────── -->
        <template v-if="tab === 'cloud'">
          <div class="max-w-lg space-y-4">
            <div>
              <label class="form-label">Google Apps Script Web App URL</label>
              <input
                v-model="gasUrl"
                class="form-input"
                placeholder="https://script.google.com/macros/s/.../exec"
              />
              <p class="text-xs text-gray-600 mt-1">部署 GAS 後取得的 Web App URL，用於雲端同步（Google Sheets）</p>
            </div>
            <button class="btn-primary text-sm" :disabled="cloudSaved" @click="saveGasUrl">
              <i v-if="cloudSaved" class="fa-solid fa-check mr-1.5 text-green-300"></i>
              {{ cloudSaved ? '已儲存' : '儲存' }}
            </button>
          </div>
        </template>

        <!-- ── 更新紀錄 ──────────────────────────────────────────── -->
        <template v-if="tab === 'changelog'">
          <div class="flex items-center justify-between mb-4">
            <div>
              <span class="text-sm text-gray-300 font-medium">Smart OR Triage</span>
              <span class="ml-2 text-xs font-mono text-gray-500">目前版本 v{{ appVersion }}</span>
            </div>
            <div v-if="updater.status.value === 'available'" class="flex items-center gap-2">
              <span class="text-xs text-blue-300">新版本 <strong>v{{ updater.updateVersion.value }}</strong> 可用</span>
              <button class="text-xs px-3 py-1 rounded bg-blue-600 hover:bg-blue-500 text-white" @click="updater.status.value = 'available'">
                <i class="fa-solid fa-circle-up mr-1"></i>更新
              </button>
            </div>
            <div v-else-if="updater.status.value === 'checking'" class="text-xs text-gray-500">
              <i class="fa-solid fa-spinner animate-spin mr-1"></i>檢查更新中…
            </div>
            <div v-else-if="updater.status.value === 'up-to-date'" class="text-xs text-green-400">
              <i class="fa-solid fa-check-circle mr-1"></i>已是最新版本
            </div>
            <button v-else class="text-xs text-gray-400 hover:text-gray-200" @click="checkUpdate">
              <i class="fa-solid fa-rotate mr-1"></i>檢查更新
            </button>
          </div>

          <div class="space-y-4">
            <div
              v-for="entry in CHANGELOG" :key="entry.version"
              class="bg-gray-700/40 rounded-lg border border-gray-700 overflow-hidden"
            >
              <div class="flex items-center gap-3 px-4 py-2.5 bg-gray-700/60 border-b border-gray-700">
                <span class="text-sm font-bold font-mono text-blue-300">v{{ entry.version }}</span>
                <span class="text-xs text-gray-500">{{ entry.date }}</span>
                <span v-if="entry.version === appVersion" class="ml-auto text-[10px] px-2 py-0.5 rounded bg-blue-900/60 border border-blue-700 text-blue-300">目前版本</span>
              </div>
              <ul class="px-4 py-3 space-y-1.5">
                <li v-for="(change, i) in entry.changes" :key="i" class="flex items-start gap-2 text-xs text-gray-300">
                  <i class="fa-solid fa-circle-dot text-[8px] text-blue-500 mt-1 shrink-0"></i>
                  {{ change }}
                </li>
              </ul>
            </div>
          </div>
        </template>

      </div>

    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from "vue";
import * as XLSX from "xlsx";
import { getVersion } from "@tauri-apps/api/app";
import { useRoomsStore } from "../stores/rooms";
import { useStaffStore } from "../stores/staff";
import { useDeptRulesStore } from "../stores/deptRules";
import { useRoomShiftsDb, useStaffRosterDb, useSettings, useSelfPayDb } from "../composables/useDatabase";
import { useLogger } from "../composables/useLogger";
import { useUpdater } from "../composables/useUpdater";
import { CHANGELOG } from "../data/changelog";
import type { Room, SelfPayItem, Staff, RoomScheduleEntry, StaffCategory, StaffRosterEntry } from "../types";
import { STAFF_CATEGORY_LABELS } from "../types";

defineEmits<{ close: [] }>();

const tab = ref<"rooms" | "staff" | "depts" | "selfpay" | "display" | "cloud" | "changelog">("rooms");
const logger = useLogger();
const updater = useUpdater();
const appVersion = ref("—");

const TABS = [
  { key: "rooms",     label: "房間管理", icon: "fa-door-open" },
  { key: "staff",     label: "人員管理", icon: "fa-users" },
  { key: "depts",     label: "科別管理", icon: "fa-layer-group" },
  { key: "selfpay",   label: "自費項目", icon: "fa-receipt" },
  { key: "display",   label: "顯示設定", icon: "fa-display" },
  { key: "cloud",     label: "雲端設定", icon: "fa-cloud" },
  { key: "changelog", label: "更新紀錄", icon: "fa-clock-rotate-left" },
] as const;

async function checkUpdate() {
  await updater.checkForUpdate();
}

const CATEGORY_LABELS = STAFF_CATEGORY_LABELS;
const CATEGORIES: StaffCategory[] = ["vs", "r", "sa", "or_nurse", "intern", "cross_train"];

const roomsStore = useRoomsStore();
const staffStore = useStaffStore();
const deptRulesStore = useDeptRulesStore();
const roomShiftsDb = useRoomShiftsDb();
const staffRosterDb = useStaffRosterDb();
const { getGasUrl, setGasUrl, getAppZoom, setAppZoom, getRoomCodeMap, setRoomCodeMap, getRoomGroups, setRoomGroups } = useSettings();

// ── Dept Rules form ───────────────────────────────────────────────────────────
const PRESET_COLORS = [
  "bg-blue-800 text-blue-100",  "bg-green-800 text-green-100",
  "bg-purple-800 text-purple-100", "bg-yellow-800 text-yellow-100",
  "bg-pink-800 text-pink-100",  "bg-teal-800 text-teal-100",
  "bg-orange-800 text-orange-100", "bg-cyan-800 text-cyan-100",
  "bg-red-800 text-red-100", "bg-indigo-800 text-indigo-100",
  "bg-gray-800 text-gray-100", "bg-slate-700 text-slate-100",
];

const deptForm = reactive({
  open: false,
  isEdit: false,
  dept: "",
  priority_bonus: 0,
  preferred_rooms: [] as string[],
  color: PRESET_COLORS[0],
});

function startAddDept() {
  Object.assign(deptForm, {
    open: true,
    isEdit: false,
    dept: "",
    priority_bonus: 0,
    preferred_rooms: [],
    color: PRESET_COLORS[0],
  });
}

function startEditDept(rule: any) {
  Object.assign(deptForm, {
    open: true,
    isEdit: true,
    dept: rule.dept,
    priority_bonus: rule.priority_bonus,
    preferred_rooms: [...(rule.preferred_rooms || [])],
    color: rule.color,
  });
}

async function saveDept() {
  if (!deptForm.dept.trim()) return;
  await deptRulesStore.upsert({
    dept: deptForm.dept.trim(),
    priority_bonus: deptForm.priority_bonus,
    preferred_rooms: deptForm.preferred_rooms,
    color: deptForm.color,
  });
  deptForm.open = false;
}

// ── Room Groups ───────────────────────────────────────────────────────────────
interface RoomGroupRow { name: string; rooms: string; }
const roomGroupRows    = ref<RoomGroupRow[]>([]);
const roomGroupSaveMsg = ref<{ ok: boolean; msg: string } | null>(null);
const roomGroupsMap    = ref<Record<string, string[]>>({});

async function loadRoomGroups() {
  try {
    const raw = await getRoomGroups();
    const map: Record<string, string[]> = JSON.parse(raw || "{}");
    roomGroupsMap.value  = map;
    roomGroupRows.value  = Object.entries(map).map(([name, rooms]) => ({ name, rooms: rooms.join(", ") }));
  } catch { roomGroupRows.value = []; }
}

function addGroupRow() { roomGroupRows.value.push({ name: "", rooms: "" }); }
function removeGroupRow(i: number) { roomGroupRows.value.splice(i, 1); }

async function saveRoomGroupRows() {
  const map: Record<string, string[]> = {};
  for (const { name, rooms } of roomGroupRows.value) {
    const k = name.trim();
    const v = rooms.split(/[,，]/).map(r => r.trim()).filter(Boolean);
    if (k && v.length) map[k] = v;
  }
  try {
    await setRoomGroups(JSON.stringify(map));
    roomGroupsMap.value = map;
    roomGroupSaveMsg.value = { ok: true, msg: "✓ 已儲存" };
    setTimeout(() => { roomGroupSaveMsg.value = null; }, 2000);
  } catch (e) {
    roomGroupSaveMsg.value = { ok: false, msg: `儲存失敗：${e}` };
  }
}

function expandByGroups(entries: RoomScheduleEntry[]): RoomScheduleEntry[] {
  const groups = roomGroupsMap.value;
  if (!Object.keys(groups).length) return entries;
  const result: RoomScheduleEntry[] = [];
  for (const entry of entries) {
    const physical = groups[entry.room_name];
    if (physical && physical.length) {
      for (const roomName of physical) {
        result.push({ ...entry, id: 0, room_name: roomName });
      }
    } else {
      result.push(entry);
    }
  }
  return result;
}

// ── PDF Code Map ──────────────────────────────────────────────────────────────
interface CodeMapRow { pdfCode: string; roomName: string; }
const codeMapRows = ref<CodeMapRow[]>([]);
const codeMapSaveMsg = ref<{ ok: boolean; msg: string } | null>(null);

async function loadCodeMap() {
  try {
    const raw = await getRoomCodeMap();
    const map: Record<string, string> = JSON.parse(raw || "{}");
    codeMapRows.value = Object.entries(map).map(([pdfCode, roomName]) => ({ pdfCode, roomName }));
  } catch { codeMapRows.value = []; }
}

function addCodeMapRow() { codeMapRows.value.push({ pdfCode: "", roomName: "" }); }
function removeCodeMapRow(i: number) { codeMapRows.value.splice(i, 1); }

async function saveCodeMap() {
  const map: Record<string, string> = {};
  for (const { pdfCode, roomName } of codeMapRows.value) {
    const k = pdfCode.trim().toLowerCase();
    if (k && roomName.trim()) map[k] = roomName.trim();
  }
  try {
    await setRoomCodeMap(JSON.stringify(map));
    codeMapSaveMsg.value = { ok: true, msg: "✓ 已儲存" };
    setTimeout(() => { codeMapSaveMsg.value = null; }, 2000);
  } catch (e) {
    codeMapSaveMsg.value = { ok: false, msg: `儲存失敗：${e}` };
  }
}

// ── Batch room add ────────────────────────────────────────────────────────────
const batchRoom = reactive({ open: false, text: "", saving: false });

function toggleBatchRoom() {
  batchRoom.open = !batchRoom.open;
  if (batchRoom.open) {
    batchRoom.text = "";
    roomForm.open = false;
  }
}

const batchRoomLines = computed(() =>
  batchRoom.text.split("\n").map((l) => l.trim()).filter(Boolean)
);
const existingNames = computed(() => new Set(roomsStore.rooms.map((r) => r.name)));
const batchRoomNew = computed(() => batchRoomLines.value.filter((n) => !existingNames.value.has(n)));
const batchRoomSkip = computed(() => batchRoomLines.value.filter((n) => existingNames.value.has(n)));

async function saveBatchRooms() {
  if (batchRoomNew.value.length === 0) return;
  batchRoom.saving = true;
  try {
    for (const name of batchRoomNew.value) {
      await roomsStore.add({ id: 0, name, display_order: roomsStore.rooms.length, is_backup: false, is_active: true });
    }
    batchRoom.open = false;
    batchRoom.text = "";
  } finally {
    batchRoom.saving = false;
  }
}

// ── Batch staff add ───────────────────────────────────────────────────────────
interface BatchStaffRow {
  name: string;
  staff_category: StaffCategory;
  unit: string;
}

const batchStaff = reactive<{ open: boolean; saving: boolean; rows: BatchStaffRow[] }>({
  open: false,
  saving: false,
  rows: [],
});

function toggleBatchStaff() {
  batchStaff.open = !batchStaff.open;
  if (batchStaff.open) {
    batchStaff.rows = [newBatchRow()];
    staffForm.open = false;
  }
}

function newBatchRow(): BatchStaffRow {
  return { name: "", staff_category: "or_nurse", unit: "" };
}

function categoryToDefaultRole(cat: StaffCategory): Staff["role"] {
  const map: Record<StaffCategory, Staff["role"]> = {
    vs: "VS", r: "R", sa: "SA", or_nurse: "Circ", intern: "R", cross_train: "Circ",
  };
  return map[cat];
}

function addBatchStaffRow() {
  batchStaff.rows.push(newBatchRow());
}

const validBatchStaffRows = computed(() =>
  batchStaff.rows.filter((r) => {
    if (!r.name.trim()) return false;
    if (r.staff_category === "cross_train" && !r.unit.trim()) return false;
    return true;
  }).length
);

async function saveBatchStaff() {
  const now = Math.floor(Date.now() / 1000);
  const valid = batchStaff.rows.filter((r) => {
    if (!r.name.trim()) return false;
    if (r.staff_category === "cross_train" && !r.unit.trim()) return false;
    return true;
  });
  if (valid.length === 0) return;
  batchStaff.saving = true;
  try {
    for (const row of valid) {
      await staffStore.add({
        id: 0,
        name: row.name.trim(),
        role: categoryToDefaultRole(row.staff_category),
        type: "nur",
        staff_category: row.staff_category,
        unit: row.unit.trim(),
        is_on_call: false,
        is_volunteer_extra: false,
        today_shift_start: now,
        next_day_shift_start: now + 86400,
      });
    }
    batchStaff.open = false;
    batchStaff.rows = [];
  } finally {
    batchStaff.saving = false;
  }
}

// ── Room form ─────────────────────────────────────────────────────────────────
const roomForm = reactive({
  open: false,
  id: 0,
  name: "",
  display_order: 0,
  is_backup: false,
  is_active: true,
});

function startAddRoom() {
  batchRoom.open = false;
  Object.assign(roomForm, { open: true, id: 0, name: "", display_order: roomsStore.rooms.length, is_backup: false, is_active: true });
}

function startEditRoom(r: Room) {
  Object.assign(roomForm, { open: true, id: r.id, name: r.name, display_order: r.display_order, is_backup: r.is_backup, is_active: r.is_active });
}

async function saveRoom() {
  if (!roomForm.name.trim()) return;
  const payload: Room = {
    id: roomForm.id,
    name: roomForm.name.trim(),
    display_order: roomForm.display_order,
    is_backup: roomForm.is_backup,
    is_active: roomForm.is_active,
  };
  if (roomForm.id) {
    await roomsStore.edit(payload);
  } else {
    await roomsStore.add(payload);
  }
  roomForm.open = false;
}

async function deleteRoom(id: number) {
  if (!confirm("確定刪除此房間？")) return;
  await roomsStore.remove(id);
}

// ── XLSX import ───────────────────────────────────────────────────────────────
const xlsxInput = ref<HTMLInputElement | null>(null);
const xlsxPreview = ref<RoomScheduleEntry[]>([]);
const xlsxErrors = ref<string[]>([]);
const xlsxMonth = ref("");
const importSaving = ref(false);

function triggerXlsx() {
  xlsxInput.value?.click();
}

function onXlsxFile(e: Event) {
  const file = (e.target as HTMLInputElement).files?.[0];
  if (!file) return;
  logger.addLog("info", `[XLSX] 選擇檔案: ${file.name} (${file.size} bytes)`);
  const reader = new FileReader();
  reader.onload = (ev) => {
    try {
      const data = new Uint8Array(ev.target!.result as ArrayBuffer);
      parseXlsx(data);
    } catch (err) {
      logger.addLog("error", `[XLSX] 讀取過程錯誤: ${err}`);
    }
    (e.target as HTMLInputElement).value = "";
  };
  reader.onerror = () => logger.addLog("error", "[XLSX] FileReader 讀取失敗");
  reader.readAsArrayBuffer(file);
}

// AM: 07:30~12:30 | PM: 12:30~15:50 | 值班: 15:50~隔日07:30 (seconds from midnight)
const PERIOD_TIMES: Record<string, { start: number; end: number }> = {
  AM: { start: 7 * 3600 + 30 * 60, end: 12 * 3600 + 30 * 60 },
  PM: { start: 12 * 3600 + 30 * 60, end: 15 * 3600 + 50 * 60 },
  值班: { start: 15 * 3600 + 50 * 60, end: 31 * 3600 + 30 * 60 },
};

function excelSerialToDateStr(serial: number): string {
  // Excel serial = days since 1899-12-30 (accounting for Lotus leap-year bug)
  const date = new Date(Math.round((serial - 25569) * 86400 * 1000));
  const y = date.getUTCFullYear();
  const m = String(date.getUTCMonth() + 1).padStart(2, "0");
  const d = String(date.getUTCDate()).padStart(2, "0");
  return `${y}-${m}-${d}`;
}

function parseXlsx(data: Uint8Array) {
  logger.addLog("info", "[XLSX] 開始解析刀房年度排班（樞紐格式）...");
  try {
    const wb = XLSX.read(data, { type: "array", raw: true, cellStyles: true });
    logger.addLog("info", `[XLSX] 找到 ${wb.SheetNames.length} 個分頁: ${wb.SheetNames.join(", ")}`);

    const entries: RoomScheduleEntry[] = [];

    for (const sheetName of wb.SheetNames) {
      const ws = wb.Sheets[sheetName];
      const range = XLSX.utils.decode_range(ws['!ref'] || 'A1');
      const rows: any[][] = XLSX.utils.sheet_to_json(ws, { header: 1, defval: "", raw: true });

      let weekDates: (string | null)[] = [];
      let currentRoom = "";

      for (let rowIdx = 0; rowIdx < rows.length; rowIdx++) {
        const row = rows[rowIdx];
        const sheetRow = range.s.r + rowIdx;
        const col0 = String(row[0] ?? "").trim();
        const col1 = String(row[1] ?? "").trim().toUpperCase();

        // Week header row (第N週): columns 2-8 hold Excel serial dates for Mon-Sun
        if (col0.startsWith("第") && col0.endsWith("週")) {
          weekDates = (row.slice(2, 9) as any[]).map((v) => {
            const n = typeof v === "number" ? v : parseFloat(String(v));
            return isNaN(n) || n < 1 ? null : excelSerialToDateStr(n);
          });
          continue;
        }

        // Track current room name (col A non-empty = new room)
        if (col0 !== "") currentRoom = col0;

        // Data row: col B = AM | PM | 值班 / 值班時段
        const periodKey = col1 === "AM" || col1 === "PM" ? col1
          : (col1 === "值班" || col1 === "值班時段" ? "值班" : null);
        if (periodKey && currentRoom && weekDates.length > 0) {
          const times = PERIOD_TIMES[periodKey];
          for (let i = 0; i < 7; i++) {
            const dept = String(row[i + 2] ?? "").trim();
            const dateStr = weekDates[i];
            if (!dept || !dateStr) continue;
            const cellAddr = XLSX.utils.encode_cell({ r: sheetRow, c: range.s.c + i + 2 });
            const isEmergency = ws[cellAddr]?.s?.fgColor?.rgb === 'FFFF00';
            entries.push({
              id: 0,
              room_name: currentRoom,
              dept,
              date: dateStr,
              start_time: dateToMidnightUnix(dateStr) + times.start,
              end_time:   dateToMidnightUnix(dateStr) + times.end,
              notes: "",
              is_emergency: isEmergency,
            });
          }
        }
      }
      logger.addLog("info", `[XLSX] 分頁 ${sheetName}: 目前累計 ${entries.length} 筆`);
    }

    if (entries.length === 0) {
      logger.addLog("error", "[XLSX] 解析失敗：未找到資料，請確認格式正確（週次、時段、星期一~日）。");
    } else {
      const months = [...new Set(entries.map(e => e.date.slice(0, 7)))].sort();
      const year = entries[0].date.slice(0, 4);
      xlsxMonth.value = months.length >= 12 ? `${year} 整年度` : months.join("、");
      logger.addLog("info", `[XLSX] 成功解析 ${entries.length} 筆，涵蓋 ${months.length} 個月份`);
    }

    xlsxPreview.value = entries;
    xlsxErrors.value  = [];
  } catch (err) {
    logger.addLog("error", `[XLSX] 檔案讀取崩潰: ${err}`);
  }
}

// Excel 時間可能是 "HH:MM" 字串，或 0.0~1.0 的日期序列小數
function normalizeExcelTime(val: any): string {
  if (!val && val !== 0) return "";
  const s = String(val).trim();
  const strMatch = s.match(/^(\d{1,2}):(\d{2})/);
  if (strMatch) return `${strMatch[1].padStart(2, "0")}:${strMatch[2]}`;
  const n = parseFloat(s);
  if (!isNaN(n) && n >= 0 && n < 1) {
    const mins = Math.round(n * 1440);
    return `${String(Math.floor(mins / 60)).padStart(2, "0")}:${String(mins % 60).padStart(2, "0")}`;
  }
  return "";
}

function parseExcelDate(val: any): string | null {
  if (!val) return null;
  if (val instanceof Date) {
    return `${val.getFullYear()}-${String(val.getMonth() + 1).padStart(2, "0")}-${String(val.getDate()).padStart(2, "0")}`;
  }
  const s = String(val);
  // YYYYMMDD (no separators, e.g. 20260501)
  let m = s.match(/^(\d{4})(\d{2})(\d{2})$/);
  if (m) return `${m[1]}-${m[2]}-${m[3]}`;
  // YYYY/MM/DD or YYYY-MM-DD
  m = s.match(/(\d{4})[\/\-](\d{1,2})[\/\-](\d{1,2})/);
  if (m) return `${m[1]}-${m[2].padStart(2, "0")}-${m[3].padStart(2, "0")}`;
  // MM/DD/YYYY
  m = s.match(/(\d{1,2})\/(\d{1,2})\/(\d{4})/);
  if (m) return `${m[3]}-${m[1].padStart(2, "0")}-${m[2].padStart(2, "0")}`;
  return null;
}


function dateToMidnightUnix(dateStr: string): number {
  return Math.floor(new Date(dateStr + "T00:00:00").getTime() / 1000);
}

function fmtTs(ts: number): string {
  const d = new Date(ts * 1000);
  return `${String(d.getHours()).padStart(2, "0")}:${String(d.getMinutes()).padStart(2, "0")}`;
}

async function confirmXlsxImport() {
  if (xlsxPreview.value.length === 0) return;
  importSaving.value = true;
  const expanded = expandByGroups(xlsxPreview.value);
  logger.addLog("info", `[XLSX] 開始寫入年度資料，原始 ${xlsxPreview.value.length} 筆，展開後 ${expanded.length} 筆...`);

  try {
    // 1. 儲存時段資料 (後端已優化為自動跨月處理)
    await roomShiftsDb.replaceByMonth("", expanded);

    // 2. 批次補建房間（使用展開後清單，避免建立虛擬群組名房間）
    const uniqueRooms = [...new Set(expanded.map(e => e.room_name))];
    const currentRoomNames = new Set(roomsStore.rooms.map(r => r.name));
    for (const name of uniqueRooms) {
      if (!currentRoomNames.has(name)) {
        await roomsStore.add({ id: 0, name, display_order: roomsStore.rooms.length, is_backup: false, is_active: true });
      }
    }

    // 3. 批次補建科別規則 (優化：先整理出不重複科別)
    const uniqueDepts = [...new Set(expanded.map(e => e.dept))].filter(Boolean);
    const currentDeptNames = new Set(deptRulesStore.rules.map(r => r.dept));
    for (const deptName of uniqueDepts) {
      if (!currentDeptNames.has(deptName)) {
        await deptRulesStore.upsert({
          dept: deptName,
          priority_bonus: 0,
          preferred_rooms: [],
          color: PRESET_COLORS[Math.floor(Math.random() * PRESET_COLORS.length)],
        });
      }
    }

    logger.addLog("info", "[XLSX] 匯入成功，正在刷新設定...");
    await Promise.all([roomsStore.load(), deptRulesStore.load()]);
    
    xlsxPreview.value = [];
    xlsxErrors.value = [];
    alert(`整年度資料匯入成功！\n- 共計 ${uniqueRooms.length} 間房間\n- 共計 ${uniqueDepts.length} 個科別設定\n\n系統已自動同步所有設定。`);
  } catch (err) {
    logger.addLog("error", `[XLSX] 寫入失敗: ${err}`);
    alert("寫入資料庫失敗，請查看 Debug Panel");
  } finally {
    importSaving.value = false;
  }
}

// ── Staff assignment xlsx import ──────────────────────────────────────────────
const staffXlsxInput   = ref<HTMLInputElement | null>(null);
const staffXlsxPreview = ref<StaffRosterEntry[]>([]);
const staffXlsxErrors  = ref<string[]>([]);
const staffXlsxMonth   = ref("");
const staffXlsxSaving  = ref(false);

// ... (省略中間 helper) ...

function parseStaffXlsx(data: Uint8Array) {
  logger.addLog("info", "[Staff XLSX] 開始解析檔案...");
  try {
    const wb = XLSX.read(data, { type: "array", cellDates: false });

    // 優先使用「條列式班表」分頁
    const sheetName = wb.SheetNames.find(n => n.includes("條列式")) ?? wb.SheetNames[0];
    logger.addLog("info", `[Staff XLSX] 讀取分頁: ${sheetName}`);
    const ws = wb.Sheets[sheetName];
    const rows: Record<string, any>[] = XLSX.utils.sheet_to_json(ws, { raw: false, defval: "" });

    logger.addLog("info", `[Staff XLSX] 讀取到 ${rows.length} 行資料`);
    if (rows.length > 0) {
      logger.addLog("info", `[Staff XLSX] 欄位範例: ${Object.keys(rows[0]).slice(0, 8).join(", ")}`);
    }

    const entries: StaffRosterEntry[] = [];
    const errors: string[] = [];
    const seen = new Set<string>();

    for (const row of rows) {
      const name      = String(row["姓名"]    ?? "").trim();
      const category  = String(row["類別"]    ?? "").trim();
      const shiftName = String(row["類別名稱"] ?? "").trim();
      const startDate = String(row["開始日期"] ?? "").trim();

      if (!name || !shiftName || !startDate) continue;
      if (category !== "S") continue; // 只取一般班別，排除加班(T)

      const dateStr = parseExcelDate(startDate);
      if (!dateStr) { errors.push(`日期格式錯誤: ${startDate}`); continue; }

      const key = `${name}|${dateStr}`;
      if (seen.has(key)) continue; // 同人同日只保留第一筆
      seen.add(key);

      const startTime = normalizeExcelTime(row["開始時間"]);
      const endTime   = normalizeExcelTime(row["結束時間"]);

      entries.push({ id: 0, staff_name: name, date: dateStr, shift_name: shiftName, start_time: startTime, end_time: endTime });
    }

    if (entries.length === 0) {
      logger.addLog("error", "[Staff XLSX] 解析失敗：未找到符合格式的資料。請確認使用「條列式班表」分頁，並包含：姓名、類別、類別名稱、開始日期。");
    } else {
      logger.addLog("info", `[Staff XLSX] 成功解析 ${entries.length} 筆人員班表（略過 ${errors.length} 筆）`);
    }

    staffXlsxMonth.value   = entries[0]?.date?.slice(0, 7) ?? "";
    staffXlsxPreview.value = entries;
    staffXlsxErrors.value  = errors;
  } catch (err) {
    logger.addLog("error", `[Staff XLSX] 檔案讀取崩潰: ${err}`);
  }
}

async function confirmStaffXlsxImport() {
  if (!staffXlsxMonth.value || staffXlsxPreview.value.length === 0) return;
  staffXlsxSaving.value = true;
  try {
    // 1. 儲存月排班
    await staffRosterDb.replaceByMonth(staffXlsxMonth.value, staffXlsxPreview.value);

    // 2. 補建人員主檔（roster 有但 staff 表沒有的人）
    const existingNames = new Set(staffStore.staffList.map(s => s.name));
    const newNames = [...new Set(staffXlsxPreview.value.map(e => e.staff_name))]
      .filter(name => name && !existingNames.has(name));
    for (const name of newNames) {
      await staffStore.add({
        id: 0, name,
        role: "Circ", type: "nur", staff_category: "or_nurse",
        unit: "", is_on_call: false, is_volunteer_extra: false,
        today_shift_start: 0, next_day_shift_start: 0,
      });
    }

    staffXlsxPreview.value = [];
    staffXlsxErrors.value  = [];
    alert(`人員月排班匯入成功！${newNames.length > 0 ? `\n自動新增 ${newNames.length} 位人員至主檔。` : ""}`);
  } catch (err) {
    logger.addLog("error", `[Staff XLSX] 匯入失敗: ${err}`);
    alert("匯入失敗，請檢查檔案格式");
  } finally {
    staffXlsxSaving.value = false;
  }
}

function triggerStaffXlsx() { 
  logger.addLog("info", "[Staff XLSX] 觸發檔案選擇器");
  staffXlsxInput.value?.click(); 
}

function onStaffXlsxFile(e: Event) {
  const file = (e.target as HTMLInputElement).files?.[0];
  if (!file) return;
  logger.addLog("info", `[Staff XLSX] 選擇檔案: ${file.name} (${file.size} bytes)`);
  const reader = new FileReader();
  reader.onload = (ev) => {
    try {
      const data = new Uint8Array(ev.target!.result as ArrayBuffer);
      parseStaffXlsx(data);
    } catch (err) {
      logger.addLog("error", `[Staff XLSX] 讀取過程錯誤: ${err}`);
    }
    (e.target as HTMLInputElement).value = "";
  };
  reader.onerror = () => logger.addLog("error", "[Staff XLSX] FileReader 讀取失敗");
  reader.readAsArrayBuffer(file);
}

// ── Staff form ────────────────────────────────────────────────────────────────
const staffForm = reactive({
  open: false,
  id: 0,
  name: "",
  staff_category: "or_nurse" as StaffCategory,
  unit: "",
  is_on_call: false,
  is_volunteer_extra: false,
});

const staffByCategory = computed(() => {
  const map: Partial<Record<StaffCategory, Staff[]>> = {};
  for (const cat of CATEGORIES) {
    map[cat] = staffStore.staffList.filter((s) => s.staff_category === cat);
  }
  return map;
});


function categoryColor(cat: StaffCategory): string {
  return ({
    vs: "text-red-400", r: "text-blue-400", sa: "text-purple-400",
    or_nurse: "text-green-400", intern: "text-yellow-400", cross_train: "text-orange-400",
  } as Record<StaffCategory, string>)[cat] ?? "text-gray-400";
}

function startAddStaff() {
  batchStaff.open = false;
  Object.assign(staffForm, { open: true, id: 0, name: "", staff_category: "or_nurse", unit: "", is_on_call: false, is_volunteer_extra: false });
}

function startEditStaff(s: Staff) {
  Object.assign(staffForm, { open: true, id: s.id, name: s.name, staff_category: s.staff_category, unit: s.unit, is_on_call: s.is_on_call, is_volunteer_extra: s.is_volunteer_extra });
}

async function saveStaff() {
  if (!staffForm.name.trim()) return;
  const now = Math.floor(Date.now() / 1000);
  const payload: Staff = {
    id: staffForm.id,
    name: staffForm.name.trim(),
    role: categoryToDefaultRole(staffForm.staff_category),
    type: "nur",
    staff_category: staffForm.staff_category,
    unit: staffForm.unit.trim(),
    is_on_call: staffForm.is_on_call,
    is_volunteer_extra: staffForm.is_volunteer_extra,
    today_shift_start: now,
    next_day_shift_start: now + 86400,
  };
  if (staffForm.id) {
    await staffStore.edit(payload);
  } else {
    await staffStore.add(payload);
  }
  staffForm.open = false;
}

async function deleteStaff(id: number) {
  if (!confirm("確定刪除此人員？")) return;
  await staffStore.remove(id);
}

// ── Self-Pay Items ────────────────────────────────────────────────────────────
const selfPayDb = useSelfPayDb();
const selfPayItems = ref<SelfPayItem[]>([]);
const selfPayImportInput = ref<HTMLInputElement | null>(null);
const selfPayForm = reactive({
  open: false,
  id: 0,
  name: "",
  price: 0,
  notes: "",
});
const selfPayImport = reactive<{
  rows: { name: string; price: number; notes: string }[];
  errors: string[];
  saving: boolean;
}>({ rows: [], errors: [], saving: false });

async function loadSelfPay() {
  selfPayItems.value = await selfPayDb.getAll();
}

function startAddSelfPay() {
  Object.assign(selfPayForm, { open: true, id: 0, name: "", price: 0, notes: "" });
}

function startEditSelfPay(item: SelfPayItem) {
  Object.assign(selfPayForm, { open: true, id: item.id, name: item.name, price: item.price, notes: item.notes });
}

async function saveSelfPay() {
  if (!selfPayForm.name.trim()) return;
  const payload: SelfPayItem = { id: selfPayForm.id, name: selfPayForm.name.trim(), price: selfPayForm.price, notes: selfPayForm.notes.trim() };
  if (selfPayForm.id) {
    const updated = await selfPayDb.update(payload);
    const idx = selfPayItems.value.findIndex(i => i.id === updated.id);
    if (idx !== -1) selfPayItems.value[idx] = updated;
  } else {
    const created = await selfPayDb.create(payload);
    selfPayItems.value.push(created);
    selfPayItems.value.sort((a, b) => a.name.localeCompare(b.name));
  }
  selfPayForm.open = false;
}

async function deleteSelfPay(id: number) {
  if (!confirm("確定刪除此自費項目？")) return;
  await selfPayDb.remove(id);
  selfPayItems.value = selfPayItems.value.filter(i => i.id !== id);
}

function downloadSelfPayTemplate() {
  const header = "name,price,notes";
  const examples = [
    "自費手套,200,乳膠手套 7.5 號",
    "自費縫線,500,",
    "自費電刀頭,1200,特殊規格",
  ];
  const csv = [header, ...examples].join("\r\n");
  const blob = new Blob(["﻿" + csv], { type: "text/csv;charset=utf-8" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = "selfpay_template.csv";
  a.click();
  URL.revokeObjectURL(url);
}

function onSelfPayCsvFile(e: Event) {
  const file = (e.target as HTMLInputElement).files?.[0];
  if (!file) return;
  const reader = new FileReader();
  reader.onload = (ev) => parseSelfPayCsv(ev.target?.result as string);
  reader.readAsText(file, "utf-8");
  (e.target as HTMLInputElement).value = "";
}

function parseSelfPayCsv(text: string) {
  selfPayImport.rows = [];
  selfPayImport.errors = [];
  const lines = text.replace(/^﻿/, "").split(/\r?\n/).filter(l => l.trim());
  if (lines.length < 2) { selfPayImport.errors.push("檔案不足兩行"); return; }
  const header = lines[0].split(",").map(h => h.trim().toLowerCase());
  const nameIdx  = header.indexOf("name");
  const priceIdx = header.indexOf("price");
  const notesIdx = header.indexOf("notes");
  if (nameIdx === -1) { selfPayImport.errors.push("找不到 name 欄位"); return; }
  lines.slice(1).forEach((line, i) => {
    const cols = line.split(",").map(c => c.trim().replace(/^"|"$/g, ""));
    const name = cols[nameIdx]?.trim() || "";
    if (!name) { selfPayImport.errors.push(`第 ${i + 2} 行：空白名稱，略過`); return; }
    const price = priceIdx !== -1 ? (parseInt(cols[priceIdx] || "0") || 0) : 0;
    const notes = notesIdx !== -1 ? (cols[notesIdx]?.trim() || "") : "";
    selfPayImport.rows.push({ name, price, notes });
  });
}

async function confirmSelfPayImport() {
  if (selfPayImport.rows.length === 0) return;
  selfPayImport.saving = true;
  try {
    for (const row of selfPayImport.rows) {
      const created = await selfPayDb.create({ id: 0, name: row.name, price: row.price, notes: row.notes });
      selfPayItems.value.push(created);
    }
    selfPayItems.value.sort((a, b) => a.name.localeCompare(b.name));
    selfPayImport.rows = [];
    selfPayImport.errors = [];
  } catch (e) {
    console.error("[SettingsModal] 自費匯入失敗:", e);
  } finally {
    selfPayImport.saving = false;
  }
}

// ── Display settings ──────────────────────────────────────────────────────────
const ZOOM_PRESETS = [0.8, 1.0, 1.3, 1.5, 1.8];
const zoomValue = ref(1.3);

async function applyZoom(zoom: number) {
  zoomValue.value = zoom;
  document.documentElement.style.setProperty('--app-zoom', zoom.toString());
  await setAppZoom(zoom);
}

// ── Cloud settings ────────────────────────────────────────────────────────────
const gasUrl = ref("");
const cloudSaved = ref(false);

async function saveGasUrl() {
  await setGasUrl(gasUrl.value.trim());
  cloudSaved.value = true;
  setTimeout(() => (cloudSaved.value = false), 2000);
}

onMounted(async () => {
  await Promise.all([roomsStore.load(), staffStore.load(), deptRulesStore.load(), loadSelfPay()]);
  const [url, zoom] = await Promise.all([getGasUrl(), getAppZoom()]);
  if (url) gasUrl.value = url;
  zoomValue.value = zoom;
  appVersion.value = await getVersion().catch(() => "—");
  await loadCodeMap();
  await loadRoomGroups();
});
</script>
