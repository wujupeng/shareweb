<template>
  <el-container class="file-browser">
    <el-header class="toolbar">
      <div class="toolbar-left">
        <el-breadcrumb separator="/">
          <el-breadcrumb-item v-for="item in breadcrumbs" :key="item.path">
            <a @click="navigateTo(item.path)">{{ item.name }}</a>
          </el-breadcrumb-item>
        </el-breadcrumb>
      </div>
      <div class="toolbar-right">
        <el-input v-model="searchKeyword" placeholder="搜索文件..." style="width: 200px" clearable @keyup.enter="doSearch">
          <template #append>
            <el-button @click="doSearch">搜索</el-button>
          </template>
        </el-input>
        <el-button-group>
          <el-button :type="viewMode === 'list' ? 'primary' : ''" @click="viewMode = 'list'">
            <el-icon><List /></el-icon>
          </el-button>
          <el-button :type="viewMode === 'grid' ? 'primary' : ''" @click="viewMode = 'grid'">
            <el-icon><Grid /></el-icon>
          </el-button>
        </el-button-group>
        <el-button type="primary" @click="showUploadDialog = true">上传</el-button>
        <el-button @click="showMkdirDialog = true">新建文件夹</el-button>
        <el-dropdown v-if="authStore.isAdmin()">
          <el-button>管理<el-icon><ArrowDown /></el-icon></el-button>
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item @click="$router.push('/admin/users')">用户管理</el-dropdown-item>
              <el-dropdown-item @click="$router.push('/admin/permissions')">权限管理</el-dropdown-item>
              <el-dropdown-item @click="$router.push('/admin/audit')">审计日志</el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
        <el-button @click="handleLogout">退出</el-button>
      </div>
    </el-header>
    <el-container>
      <el-aside width="240px" class="sidebar">
        <el-tree :data="treeData" :props="treeProps" lazy :load="loadTreeNode" @node-click="onTreeNodeClick" highlight-current node-key="path" />
      </el-aside>
      <el-main class="file-main">
        <el-table v-if="viewMode === 'list'" :data="fileList" @row-dblclick="onRowDblClick" @row-contextmenu="onRowContextMenu" stripe>
          <el-table-column prop="name" label="名称" min-width="300">
            <template #default="{ row }">
              <el-icon v-if="row.is_dir" style="color: #409eff"><Folder /></el-icon>
              <el-icon v-else><Document /></el-icon>
              <span style="margin-left: 8px; cursor: pointer">{{ row.name }}</span>
            </template>
          </el-table-column>
          <el-table-column prop="size" label="大小" width="120">
            <template #default="{ row }">{{ row.is_dir ? '-' : formatSize(row.size) }}</template>
          </el-table-column>
          <el-table-column prop="modified" label="修改时间" width="180" />
          <el-table-column label="操作" width="250">
            <template #default="{ row }">
              <el-button size="small" @click="previewFile(row)" v-if="!row.is_dir && row.preview_type !== 'none'">预览</el-button>
              <el-button size="small" @click="downloadFile(row)" v-if="!row.is_dir">下载</el-button>
              <el-button size="small" @click="showRenameDialog(row)">重命名</el-button>
              <el-button size="small" type="danger" @click="confirmDelete(row)">删除</el-button>
            </template>
          </el-table-column>
        </el-table>
        <div v-else class="grid-view">
          <div v-for="file in fileList" :key="file.path" class="grid-item" @dblclick="onRowDblClick(file)" @contextmenu.prevent="onRowContextMenu(file, $event)">
            <el-icon :size="48" :color="file.is_dir ? '#409eff' : '#909399'">
              <Folder v-if="file.is_dir" /><Document v-else />
            </el-icon>
            <div class="grid-name">{{ file.name }}</div>
          </div>
        </div>
      </el-main>
    </el-container>

    <el-dialog v-model="showMkdirDialog" title="新建文件夹" width="400px">
      <el-input v-model="newFolderName" placeholder="请输入文件夹名称" />
      <template #footer>
        <el-button @click="showMkdirDialog = false">取消</el-button>
        <el-button type="primary" @click="createFolder">确定</el-button>
      </template>
    </el-dialog>

    <el-dialog v-model="showRenameDialogVisible" title="重命名" width="400px">
      <el-input v-model="renameNewName" placeholder="请输入新名称" />
      <template #footer>
        <el-button @click="showRenameDialogVisible = false">取消</el-button>
        <el-button type="primary" @click="doRename">确定</el-button>
      </template>
    </el-dialog>

    <el-dialog v-model="showUploadDialog" title="上传文件" width="500px">
      <el-upload drag multiple :auto-upload="false" :on-change="onFileChange" :file-list="uploadFileList">
        <el-icon :size="64"><Upload /></el-icon>
        <div>拖拽文件到此处或点击上传</div>
      </el-upload>
      <template #footer>
        <el-button @click="showUploadDialog = false">取消</el-button>
        <el-button type="primary" @click="startUpload" :loading="uploading">上传</el-button>
      </template>
    </el-dialog>

    <el-dialog v-model="showPreviewDialog" :title="previewFileName" width="80%" top="5vh">
      <div class="preview-container">
        <img v-if="previewType === 'image'" :src="previewUrl" style="max-width: 100%; max-height: 70vh" />
        <video v-else-if="previewType === 'video'" :src="previewUrl" controls style="max-width: 100%; max-height: 70vh" />
        <audio v-else-if="previewType === 'audio'" :src="previewUrl" controls />
        <iframe v-else-if="previewType === 'pdf'" :src="previewUrl" style="width: 100%; height: 70vh" />
        <pre v-else-if="previewType === 'text'" class="text-preview">{{ textContent }}</pre>
        <div v-else>不支持预览此文件类型</div>
      </div>
    </el-dialog>
  </el-container>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useAuthStore } from '@/stores/auth'
import { filesApi, type FileInfo } from '@/api/files'
import { uploadApi } from '@/api/upload'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Folder, Document, List, Grid, ArrowDown, Upload } from '@element-plus/icons-vue'

const authStore = useAuthStore()
const currentPath = ref('/')
const fileList = ref<FileInfo[]>([])
const treeData = ref<any[]>([])
const viewMode = ref<'list' | 'grid'>('list')
const searchKeyword = ref('')

const showMkdirDialog = ref(false)
const newFolderName = ref('')
const showRenameDialogVisible = ref(false)
const renameNewName = ref('')
const renameTarget = ref<FileInfo | null>(null)
const showUploadDialog = ref(false)
const uploadFileList = ref<any[]>([])
const uploading = ref(false)

const showPreviewDialog = ref(false)
const previewType = ref('')
const previewUrl = ref('')
const previewFileName = ref('')
const textContent = ref('')

const treeProps = { label: 'name', children: 'children', isLeaf: (data: any) => !data.has_children }

const breadcrumbs = computed(() => {
  const parts = currentPath.value.split('/').filter(Boolean)
  const crumbs = [{ name: '根目录', path: '/' }]
  let path = ''
  for (const part of parts) {
    path += '/' + part
    crumbs.push({ name: part, path })
  }
  return crumbs
})

async function loadFiles() {
  try {
    const res = await filesApi.listFiles({ path: currentPath.value, sort_by: 'type', sort_order: 'asc' })
    fileList.value = res.data.data.items
  } catch (e: any) {
    ElMessage.error('加载文件列表失败: ' + (e.message || ''))
  }
}

async function loadTreeNode(node: any, resolve: Function) {
  const path = node.level === 0 ? '/' : node.data.path
  try {
    const res = await filesApi.getTree(path)
    resolve(res.data.data || [])
  } catch {
    resolve([])
  }
}

function onTreeNodeClick(data: any) {
  navigateTo(data.path)
}

async function navigateTo(path: string) {
  currentPath.value = path
  await loadFiles()
}

function onRowDblClick(row: FileInfo) {
  if (row.is_dir) {
    navigateTo(row.path)
  } else {
    previewFile(row)
  }
}

function onRowContextMenu(_row: FileInfo, _event: MouseEvent) {}

async function doSearch() {
  if (!searchKeyword.value) { await loadFiles(); return }
  try {
    const res = await filesApi.search(searchKeyword.value, currentPath.value)
    fileList.value = res.data.data
  } catch (e: any) {
    ElMessage.error('搜索失败: ' + (e.message || ''))
  }
}

async function createFolder() {
  if (!newFolderName.value) return
  try {
    await filesApi.mkdir(currentPath.value, newFolderName.value)
    ElMessage.success('文件夹创建成功')
    showMkdirDialog.value = false
    newFolderName.value = ''
    await loadFiles()
  } catch (e: any) {
    ElMessage.error(e.message || '创建失败')
  }
}

function showRenameDialog(file: FileInfo) {
  renameTarget.value = file
  renameNewName.value = file.name
  showRenameDialogVisible.value = true
}

async function doRename() {
  if (!renameTarget.value || !renameNewName.value) return
  try {
    await filesApi.rename(renameTarget.value.path, renameNewName.value)
    ElMessage.success('重命名成功')
    showRenameDialogVisible.value = false
    await loadFiles()
  } catch (e: any) {
    ElMessage.error(e.message || '重命名失败')
  }
}

async function confirmDelete(file: FileInfo) {
  try {
    await ElMessageBox.confirm(`确定删除 ${file.name} 吗？`, '确认删除', { type: 'warning' })
    await filesApi.delete(file.path)
    ElMessage.success('删除成功')
    await loadFiles()
  } catch { /* cancelled */ }
}

function downloadFile(file: FileInfo) {
  window.open(filesApi.download(file.path), '_blank')
}

async function previewFile(file: FileInfo) {
  if (!file.preview_type || file.preview_type === 'none') {
    ElMessage.info('不支持预览，请下载查看')
    return
  }
  previewFileName.value = file.name
  previewType.value = file.preview_type
  if (file.preview_type === 'text') {
    try {
      const token = authStore.token
      const resp = await fetch(`/api/files/preview?path=${encodeURIComponent(file.path)}`, {
        headers: { Authorization: `Bearer ${token}` }
      })
      const json = await resp.json()
      textContent.value = json.data?.content || ''
      showPreviewDialog.value = true
    } catch {
      ElMessage.error('预览失败')
    }
  } else {
    previewUrl.value = filesApi.preview(file.path)
    showPreviewDialog.value = true
  }
}

function onFileChange(_file: any, fileListNew: any[]) {
  uploadFileList.value = fileListNew
}

async function startUpload() {
  if (!uploadFileList.value.length) return
  uploading.value = true
  try {
    for (const file of uploadFileList.value) {
      const raw = file.raw as File
      const totalSize = raw.size
      const chunkSize = 5 * 1024 * 1024

      const initRes = await uploadApi.init({
        file_name: raw.name,
        target_path: currentPath.value,
        total_size: totalSize,
      })
      const { task_id, total_chunks } = initRes.data.data
      for (let i = 0; i < total_chunks; i++) {
        const start = i * chunkSize
        const end = Math.min(start + chunkSize, totalSize)
        const chunk = raw.slice(start, end)
        const arrayBuf = await chunk.arrayBuffer()
        await uploadApi.uploadChunk(task_id, i, arrayBuf)
      }
      await uploadApi.complete({
        task_id,
        file_name: raw.name,
        target_path: currentPath.value,
        total_chunks,
      })
    }
    ElMessage.success('上传完成')
    showUploadDialog.value = false
    uploadFileList.value = []
    await loadFiles()
  } catch (e: any) {
    ElMessage.error('上传失败: ' + (e.message || ''))
  } finally {
    uploading.value = false
  }
}

function handleLogout() {
  authStore.logout()
  location.href = '/login'
}

function formatSize(bytes: number): string {
  if (bytes === 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(1024))
  return (bytes / Math.pow(1024, i)).toFixed(1) + ' ' + units[i]
}

onMounted(() => {
  loadFiles()
})
</script>

<style scoped>
.file-browser { height: 100vh; }
.toolbar { display: flex; align-items: center; justify-content: space-between; padding: 8px 16px; border-bottom: 1px solid #e4e7ed; }
.toolbar-right { display: flex; gap: 8px; align-items: center; }
.sidebar { border-right: 1px solid #e4e7ed; overflow-y: auto; }
.file-main { padding: 16px; overflow-y: auto; }
.grid-view { display: flex; flex-wrap: wrap; gap: 16px; }
.grid-item { width: 100px; text-align: center; cursor: pointer; padding: 8px; border-radius: 4px; }
.grid-item:hover { background: #f5f7fa; }
.grid-name { margin-top: 4px; font-size: 12px; word-break: break-all; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.preview-container { text-align: center; }
.text-preview { text-align: left; max-height: 70vh; overflow: auto; background: #1e1e1e; color: #d4d4d4; padding: 16px; border-radius: 4px; font-size: 14px; }
</style>
