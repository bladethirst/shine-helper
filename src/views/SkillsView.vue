<template>
  <div class="h-full flex flex-col bg-gray-50">
    <div class="px-6 pt-6 pb-0">
      <div class="flex items-center justify-between mb-4">
        <div>
          <h1 class="text-2xl font-bold text-gray-800">Skills</h1>
          <p class="text-sm text-gray-500 mt-0.5">{{ T.subtitle }}</p>
        </div>
        <button @click="activeTab==='market'?fetchMarketSkills():loadLocalSkills()" :disabled="loading"
          class="px-4 py-2 text-primary-600 border border-primary-600 rounded-lg hover:bg-primary-50 disabled:opacity-50 text-sm">{{ T.refresh }}</button>
      </div>
      <div class="flex border-b border-gray-200">
        <button @click="activeTab='installed'" class="px-5 py-2.5 text-sm font-medium border-b-2 transition-colors"
          :class="activeTab==='installed'?'border-primary-500 text-primary-600':'border-transparent text-gray-500'">
          {{ T.tabInstalled }}<span v-if="installedSkills.length>0" class="ml-1.5 px-1.5 py-0.5 text-xs rounded-full bg-gray-100">{{ installedSkills.length }}</span>
        </button>
        <button @click="activeTab='market'" class="px-5 py-2.5 text-sm font-medium border-b-2 transition-colors"
          :class="activeTab==='market'?'border-primary-500 text-primary-600':'border-transparent text-gray-500'">
          {{ T.tabMarket }}<span v-if="marketSkills.length>0" class="ml-1.5 px-1.5 py-0.5 text-xs rounded-full bg-gray-100">{{ marketSkills.length }}</span>
        </button>
      </div>
    </div>
    <div class="flex-1 overflow-auto px-6 py-5">
      <div v-if="activeTab==='installed'">
        <div v-if="installedSkills.length===0" class="text-center py-20">
          <p class="text-gray-500 font-medium">{{ T.noInstalled }}</p>
          <button @click="activeTab='market'" class="mt-4 px-4 py-2 text-sm bg-primary-500 text-white rounded-lg">{{ T.goMarket }}</button>
        </div>
        <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          <div v-for="s in installedSkills" :key="s.id" @click="openInstalledDetail(s)"
            class="bg-white border border-gray-100 rounded-xl p-4 shadow-sm hover:shadow-md hover:border-primary-200 transition-all flex flex-col cursor-pointer">
            <div class="flex items-start gap-3 mb-2">
              <div class="w-10 h-10 rounded-lg bg-primary-50 flex items-center justify-center text-xl flex-shrink-0">{{ getSkillIcon(s.name) }}</div>
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2"><h3 class="font-semibold text-gray-800 truncate">{{ s.name }}</h3><span class="text-xs bg-green-100 text-green-700 px-1.5 py-0.5 rounded-full">{{ T.badgeInstalled }}</span></div>
                <span v-if="s.installed_version || s.version" class="text-xs text-primary-500">v{{ s.installed_version || s.version }}</span>
              </div>
            </div>
            <p class="text-sm text-gray-500 line-clamp-2 mb-3 flex-1">{{ s.description || T.noDesc }}</p>
            <div class="flex items-center justify-between pt-2 border-t border-gray-50">
              <span class="text-xs text-gray-400">{{ s.file_name || s.id }}</span>
              <button @click.stop="uninstallSkill(s.id)" class="px-3 py-1.5 text-xs text-red-600 border border-red-200 rounded-lg hover:bg-red-50">{{ T.uninstall }}</button>
            </div>
          </div>
        </div>

      </div>
      <div v-if="activeTab==='market'">
        <div class="flex items-center justify-between gap-4 mb-5">
          <input v-model="searchQuery" type="text" :placeholder="T.searchPH" class="flex-1 px-4 py-2.5 border border-gray-200 rounded-lg bg-white focus:outline-none focus:ring-2 focus:ring-primary-400 text-sm" />
          <select v-model="selectedCategory" class="px-4 py-2.5 border border-gray-200 rounded-lg bg-white focus:outline-none focus:ring-2 focus:ring-primary-400 text-sm min-w-[120px]">
            <option value="">{{ T.allCategories }}</option>
            <option v-for="cat in categories" :key="cat" :value="cat">{{ cat }}</option>
          </select>
        </div>
        <div v-if="error" class="mb-5 p-4 bg-red-50 border border-red-200 rounded-lg flex items-center gap-3">
          <div><p class="text-sm font-medium text-red-700">{{ T.loadFail }}</p><p class="text-xs text-red-500">{{ error }}</p></div>
          <button @click="fetchMarketSkills" class="ml-auto text-xs text-red-600 underline">{{ T.retry }}</button>
        </div>
        <div v-if="loading" class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          <div v-for="i in 6" :key="i" class="bg-white border border-gray-100 rounded-xl p-4 animate-pulse">
            <div class="flex gap-3 mb-3"><div class="w-10 h-10 bg-gray-200 rounded-lg"></div><div class="flex-1"><div class="h-4 bg-gray-200 rounded w-2/3 mb-1"></div><div class="h-3 bg-gray-100 rounded w-1/3"></div></div></div>
            <div class="h-3 bg-gray-100 rounded w-full mb-2"></div><div class="h-3 bg-gray-100 rounded w-4/5 mb-4"></div>
          </div>
        </div>
        <div v-else-if="filteredMarketSkills.length===0 && !error" class="text-center py-16">
          <p class="text-gray-500">{{ searchQuery ? T.noResult : T.noSkills }}</p>
        </div>
        <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          <div v-for="s in filteredMarketSkills" :key="s.id" @click="openMarketDetail(s)"
            class="bg-white border border-gray-100 rounded-xl p-4 shadow-sm hover:shadow-md hover:border-primary-200 transition-all flex flex-col cursor-pointer">
            <div class="flex items-start gap-3 mb-2">
              <div class="w-10 h-10 rounded-lg bg-primary-50 flex items-center justify-center text-xl flex-shrink-0">{{ getSkillIcon(s.name) }}</div>
              <div class="flex-1 min-w-0">
                <h3 class="font-semibold text-gray-800 truncate">{{ s.name }}</h3>
                <div class="flex items-center gap-1.5 mt-0.5"><span v-if="s.version" class="text-xs text-primary-500">v{{ s.version }}</span><span v-if="s.version" class="text-gray-300">&#xB7;</span><span class="text-xs text-gray-400">{{ s.parent_dir }}</span></div>
              </div>
              <span v-if="isInstalled(s.id)" class="text-xs bg-green-100 text-green-700 px-1.5 py-0.5 rounded-full flex-shrink-0">{{ T.badgeInstalled }}</span>
            </div>
            <p class="text-sm text-gray-500 line-clamp-2 mb-3 flex-1">{{ s.description || T.noDesc }}</p>
            <div class="flex items-center justify-between mt-auto pt-2 border-t border-gray-50">
              <div class="text-xs text-gray-400">{{ s.download_count }} {{ T.downloads }} &#xB7; {{ timeAgo(s.updated_at) }}</div>
              <button @click.stop="installSkill(s)" :disabled="installingIds.has(s.id)||isInstalled(s.id)"
                class="px-3 py-1.5 text-xs rounded-lg disabled:cursor-not-allowed"
                :class="isInstalled(s.id)?'bg-gray-100 text-gray-400':'bg-primary-500 text-white hover:bg-primary-600 disabled:opacity-60'">
                {{ installingIds.has(s.id)?T.installing:isInstalled(s.id)?T.badgeInstalled:T.install }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
    <Transition name="modal">
      <div v-if="detailModal.show" class="fixed inset-0 z-50 flex items-center justify-center p-4">
        <div class="absolute inset-0 bg-black/40 backdrop-blur-sm" @click="closeDetail"></div>
        <div class="relative bg-white rounded-2xl shadow-2xl w-full max-w-lg max-h-[80vh] flex flex-col overflow-hidden">
          <div class="flex items-start gap-4 p-6 border-b border-gray-100">
            <div class="w-14 h-14 rounded-xl bg-primary-50 flex items-center justify-center text-3xl flex-shrink-0">{{ getSkillIcon(detailModal.skill?.name||'') }}</div>
            <div class="flex-1 min-w-0">
              <h2 class="text-xl font-bold text-gray-800">{{ detailModal.skill?.name }}</h2>
              <div class="flex items-center gap-2 mt-1 flex-wrap">
                <span v-if="detailModal.skill?.version" class="text-xs text-primary-500">v{{ detailModal.skill.version }}</span>
                <span v-if="detailModal.skill?.parent_dir" class="text-xs text-gray-400 bg-gray-100 px-1.5 py-0.5 rounded">{{ detailModal.skill.parent_dir }}</span>
                <span v-if="detailModal.isInstalled" class="text-xs bg-green-100 text-green-700 px-1.5 py-0.5 rounded-full">{{ T.badgeInstalled }}</span>
              </div>
            </div>
            <button @click="closeDetail" class="text-gray-400 hover:text-gray-600 p-1">
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/></svg>
            </button>
          </div>
          <div class="flex-1 overflow-auto p-6">
            <div v-if="detailModal.loading" class="space-y-3 animate-pulse">
              <div class="h-4 bg-gray-200 rounded w-full"></div><div class="h-4 bg-gray-200 rounded w-5/6"></div>
            </div>
            <div v-else>
              <div class="mb-5">
                <h3 class="text-xs font-semibold text-gray-500 uppercase tracking-wide mb-2">{{ T.desc }}</h3>
                <p class="text-sm text-gray-700 leading-relaxed">{{ detailModal.detail?.yaml?.description_zh || detailModal.detail?.yaml?.description || detailModal.skill?.description || T.noDesc }}</p>
              </div>
              <div class="bg-gray-50 rounded-lg p-3 space-y-2 mb-5">
                <div class="flex justify-between text-sm"><span class="text-gray-500">{{ T.version }}</span><span class="font-medium text-gray-700">{{ detailModal.detail?.yaml?.version||detailModal.skill?.version||'-' }}</span></div>
                <div class="flex justify-between text-sm"><span class="text-gray-500">{{ T.fileName }}</span><span class="font-mono text-xs text-gray-700">{{ detailModal.skill?.file_name }}</span></div>
                <div v-if="detailModal.detail?.yaml?.license" class="flex justify-between text-sm"><span class="text-gray-500">{{ T.license }}</span><span class="text-gray-700">{{ detailModal.detail.yaml.license }}</span></div>
                <div v-if="detailModal.marketSkill" class="flex justify-between text-sm"><span class="text-gray-500">{{ T.dlCount }}</span><span class="text-gray-700">{{ detailModal.marketSkill.download_count }}</span></div>
                <div v-if="detailModal.marketSkill" class="flex justify-between text-sm"><span class="text-gray-500">{{ T.updatedAt }}</span><span class="text-gray-700">{{ formatDate(detailModal.marketSkill.updated_at) }}</span></div>
                <div v-if="detailModal.marketSkill" class="flex justify-between text-sm"><span class="text-gray-500">{{ T.fileSize }}</span><span class="text-gray-700">{{ formatSize(detailModal.marketSkill.size) }}</span></div>
              </div>
              <div v-if="detailModal.detail?.yaml?.description && detailModal.detail?.yaml?.description_zh">
                <h3 class="text-xs font-semibold text-gray-500 uppercase tracking-wide mb-2">English Description</h3>
                <p class="text-xs text-gray-500 leading-relaxed">{{ detailModal.detail.yaml.description }}</p>
              </div>
            </div>
          </div>
          <div class="p-6 border-t border-gray-100 flex items-center justify-between">
            <div class="text-xs text-gray-400">{{ detailModal.skill?.file_name }}</div>
            <div class="flex gap-2">
              <button @click="closeDetail" class="px-4 py-2 text-sm text-gray-600 border border-gray-200 rounded-lg hover:bg-gray-50">{{ T.close }}</button>
              <button v-if="!detailModal.isInstalled" @click="installSkillFromDetail" :disabled="detailModal.installing"
                class="px-4 py-2 text-sm bg-primary-500 text-white rounded-lg hover:bg-primary-600 disabled:opacity-60 flex items-center gap-2">
                {{ detailModal.installing ? T.installing : T.install }}
              </button>
              <button v-if="detailModal.isInstalled" @click="uninstallFromDetail"
                class="px-4 py-2 text-sm text-red-600 border border-red-200 rounded-lg hover:bg-red-50">{{ T.uninstall }}</button>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>
<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'

const T = {
  subtitle: '\u7BA1\u7406\u548C\u53D1\u73B0 AI \u6280\u80FD\u6269\u5C55',
  refresh: '\u5237\u65B0',
  tabInstalled: '\u5DF2\u5B89\u88C5',
  tabMarket: 'Skills \u5E02\u573A',
  noInstalled: '\u5C1A\u672A\u5B89\u88C5\u4EFB\u4F55 Skill',
  goMarket: '\u53BB\u5E02\u573A\u770B\u770B',
  badgeInstalled: '\u5DF2\u5B89\u88C5',
  noDesc: '\u6682\u65E0\u63CF\u8FF0',
  uninstall: '\u5378\u8F7D',
  searchPH: '\u641C\u7D22 Skills \u540D\u79F0\u6216\u63CF\u8FF0...',
  allCategories: '\u5168\u90E8\u5206\u7C7B',
  loadFail: '\u52A0\u8F7D\u5931\u8D25',
  retry: '\u91CD\u8BD5',
  noResult: '\u6CA1\u6709\u627E\u5230\u5339\u914D\u7684 Skills',
  noSkills: '\u6682\u65E0\u53EF\u7528\u7684 Skills',
  downloads: '\u4E0B\u8F7D',
  installing: '\u5B89\u88C5\u4E2D...',
  install: '\u5B89\u88C5',
  desc: '\u63CF\u8FF0',
  info: '\u57FA\u672C\u4FE1\u606F',
  version: '\u7248\u672C',
  fileName: '\u6587\u4EF6\u540D',
  license: '\u8BB8\u53EF\u8BC1',
  dlCount: '\u4E0B\u8F7D\u6B21\u6570',
  updatedAt: '\u66F4\u65B0\u65F6\u95F4',
  fileSize: '\u6587\u4EF6\u5927\u5C0F',
  close: '\u5173\u95ED',
}


interface MarketSkill {
  id: string; file_name: string; name: string; description: string
  version: string; parent_dir: string; category: string; download_count: number
  updated_at: string; sha256_hash: string; size: number
}
interface LocalSkill {
  id: string; name: string; description: string; version: string
  file_name?: string; installed: boolean; enabled: boolean; installed_version?: string
}
interface SkillDetail {
  id: string; parent_dir: string
  yaml: { name: string; version?: string; description?: string; description_zh?: string; license?: string }
}
interface DetailModal {
  show: boolean; loading: boolean; installing: boolean; isInstalled: boolean
  skill: MarketSkill | null; marketSkill: MarketSkill | null; detail: SkillDetail | null
}

const isTauri = typeof window !== 'undefined' && '__TAURI_IPC__' in window
const activeTab = ref<'installed'|'market'>('market')
const searchQuery = ref('')
const selectedCategory = ref('')
const loading = ref(false)
const error = ref<string|null>(null)
const marketSkills = ref<MarketSkill[]>([])
const installedSkills = ref<LocalSkill[]>([])
const installingIds = ref<Set<string>>(new Set())
const detailModal = ref<DetailModal>({ show:false, loading:false, installing:false, isInstalled:false, skill:null, marketSkill:null, detail:null })

function getMarketConfig() {
  try {
    const p = JSON.parse(localStorage.getItem('shine_helper_config') || '{}')
    return { url: p?.market?.url || 'http://127.0.0.1:5000', apiKey: p?.market?.api_key || '' }
  } catch { return { url: 'http://127.0.0.1:5000', apiKey: '' } }
}

async function httpGet(url: string, headers: Record<string,string> = {}) {
  if (isTauri) {
    const { fetch: tf } = await import('@tauri-apps/api/http')
    const r = await tf(url, { method: 'GET', headers })
    if (!r.ok) throw new Error(`HTTP ${r.status}`)
    return r.data
  }
  const r = await fetch(url.replace(/^https?:\/\/[^/]+/, '/api-market'), { headers })
  if (!r.ok) throw new Error(`HTTP ${r.status}`)
  return r.json()
}

async function httpGetBinary(url: string, headers: Record<string,string> = {}): Promise<Uint8Array> {
  if (isTauri) {
    const { fetch: tf, ResponseType } = await import('@tauri-apps/api/http')
    const r = await tf(url, { method: 'GET', headers, responseType: ResponseType.Binary })
    if (!r.ok) throw new Error(`HTTP ${r.status}`)
    return new Uint8Array(r.data as number[])
  }
  const r = await fetch(url.replace(/^https?:\/\/[^/]+/, '/api-market'), { headers })
  if (!r.ok) throw new Error(`HTTP ${r.status}`)
  return new Uint8Array(await r.arrayBuffer())
}

const installedIds = computed(() => new Set(installedSkills.value.map(s => s.id)))
const categories = computed(() => {
  const cats = new Set(marketSkills.value.map(s => s.category).filter(Boolean))
  return Array.from(cats).sort()
})
const filteredMarketSkills = computed(() => {
  const q = searchQuery.value.toLowerCase().trim()
  const cat = selectedCategory.value
  return marketSkills.value.filter(s => {
    const matchSearch = !q || s.name.toLowerCase().includes(q) || (s.description||'').toLowerCase().includes(q)
    const matchCategory = !cat || s.category === cat
    return matchSearch && matchCategory
  })
})
function isInstalled(id: string) { return installedIds.value.has(id) }

const ICONS: Record<string,string> = {
  pdf: String.fromCodePoint(0x1F4C4),
  pptx: String.fromCodePoint(0x1F4CA),
  ppt: String.fromCodePoint(0x1F4CA),
  excel: String.fromCodePoint(0x1F4C8),
  xlsx: String.fromCodePoint(0x1F4C8),
  word: String.fromCodePoint(0x1F4DD),
  docx: String.fromCodePoint(0x1F4DD),
  email: String.fromCodePoint(0x1F4E7),
  image: String.fromCodePoint(0x1F5BC),
  video: String.fromCodePoint(0x1F3AC),
  audio: String.fromCodePoint(0x1F3B5),
  data: String.fromCodePoint(0x1F5C4),
  web: String.fromCodePoint(0x1F310),
  sql: String.fromCodePoint(0x1F5C3),
  code: String.fromCodePoint(0x1F4BB),
  git: String.fromCodePoint(0x1F500),
  zip: String.fromCodePoint(0x1F5C2),
  report: String.fromCodePoint(0x1F4CB),
}
const DEFAULT_ICON = String.fromCodePoint(0x1F4E6)
function getSkillIcon(name: string) {
  const l = name.toLowerCase()
  for (const [k,v] of Object.entries(ICONS)) if (l.includes(k)) return v
  return DEFAULT_ICON
}
function formatDate(iso: string) { try { return new Date(iso).toLocaleString('zh-CN') } catch { return iso } }
function timeAgo(iso: string) {
  try {
    const d = Math.floor((Date.now()-new Date(iso).getTime())/86400000)
    if (d===0) return '\u4ECA\u5929'; if (d===1) return '\u6628\u5929'
    if (d<30) return `${d}\u5929\u524D`; if (d<365) return `${Math.floor(d/30)}\u4E2A\u6708\u524D`
    return `${Math.floor(d/365)}\u5E74\u524D`
  } catch { return '-' }
}
function formatSize(b: number) {
  if (b<1024) return `${b} B`
  if (b<1048576) return `${(b/1024).toFixed(1)} KB`
  return `${(b/1048576).toFixed(1)} MB`
}

async function fetchMarketSkills() {
  loading.value=true; error.value=null
  try {
    const { url, apiKey } = getMarketConfig()
    const h: Record<string,string> = {}; if (apiKey) h['X-API-Key']=apiKey
    marketSkills.value = await httpGet(`${url}/api/skills`, h) as MarketSkill[]
  } catch(e: any) { error.value = e?.message || '\u65E0\u6CD5\u8FDE\u63A5\u5230 Skills \u670D\u52A1' }
  finally { loading.value=false }
}
async function loadLocalSkills() {
  if (!isTauri) return
  try { installedSkills.value = await invoke<LocalSkill[]>('get_local_skills') }
  catch(e) { console.warn('get_local_skills:', e) }
}
onMounted(() => Promise.all([fetchMarketSkills(), loadLocalSkills(), loadSkillsDir()]))

const skillsDir = ref('')
async function loadSkillsDir() {
  if (!isTauri) return
  try { skillsDir.value = await invoke<string>('get_skills_dir') }
  catch(e) { console.warn('get_skills_dir:', e) }
}

async function openMarketDetail(skill: MarketSkill) {
  detailModal.value = { show:true, loading:true, installing:false, isInstalled:isInstalled(skill.id), skill, marketSkill:skill, detail:null }
  try {
    const { url, apiKey } = getMarketConfig()
    const h: Record<string,string> = {}; if (apiKey) h['X-API-Key']=apiKey
    detailModal.value.detail = await httpGet(`${url}/api/skill/${skill.file_name}/detail`, h) as SkillDetail
  } catch(e) { console.warn('detail:', e) }
  finally { detailModal.value.loading=false }
}
function openInstalledDetail(skill: LocalSkill) {
  const ms = marketSkills.value.find(s => s.id===skill.id)||null
  detailModal.value = {
    show:true, loading:false, installing:false, isInstalled:true,
    skill: ms||{ id:skill.id, file_name:skill.file_name||skill.id, name:skill.name, description:skill.description, version:skill.version||'', parent_dir:'', download_count:0, updated_at:'', sha256_hash:'', size:0 } as MarketSkill,
    marketSkill:ms, detail:null
  }
}
function closeDetail() { detailModal.value.show=false }

async function installSkill(skill: MarketSkill) {
  if (!isTauri) {
    alert('\u5B89\u88C5\u529F\u80FD\u9700\u8981\u5728 Tauri \u5E94\u7528\u4E2D\u4F7F\u7528')
    return
  }
  installingIds.value = new Set([...installingIds.value, skill.id])
  try {
    const { url, apiKey } = getMarketConfig()
    const h: Record<string,string> = {}; if (apiKey) h['X-API-Key']=apiKey
    const data = await httpGetBinary(`${url}/api/download/${skill.parent_dir}/${skill.file_name}`, h)
    await invoke('install_skill', { skillId:skill.id, skillName:skill.name, skillDescription:skill.description||'', skillVersion:skill.version||'', skillFileName:skill.file_name, skillData:Array.from(data) })
    await loadLocalSkills()
  } catch(e) { console.error('install:', e); alert('\u5B89\u88C5\u5931\u8D25: '+e) }
  finally { const s=new Set(installingIds.value); s.delete(skill.id); installingIds.value=s }
}
async function installSkillFromDetail() {
  if (!detailModal.value.marketSkill) return
  detailModal.value.installing=true
  await installSkill(detailModal.value.marketSkill)
  detailModal.value.installing=false
  detailModal.value.isInstalled=isInstalled(detailModal.value.marketSkill.id)
}
async function uninstallSkill(skillId: string) {
  try { await invoke('uninstall_skill', { skillId }); await loadLocalSkills() }
  catch(e) { console.error('uninstall:', e) }
}
async function uninstallFromDetail() {
  if (!detailModal.value.skill) return
  await uninstallSkill(detailModal.value.skill.id)
  detailModal.value.isInstalled=false
  closeDetail()
}
</script>

<style scoped>
.modal-enter-active,.modal-leave-active{transition:opacity .2s ease}
.modal-enter-from,.modal-leave-to{opacity:0}
</style>
