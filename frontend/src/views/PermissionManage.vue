<template>
  <el-container style="height: 100vh">
    <el-header style="display: flex; align-items: center; justify-content: space-between; border-bottom: 1px solid #e4e7ed; padding: 0 16px">
      <h2 style="margin: 0">权限管理</h2>
      <div>
        <el-button @click="$router.push('/')">返回文件浏览</el-button>
      </div>
    </el-header>
    <el-main>
      <div style="margin-bottom: 16px">
        <el-button type="primary" @click="showCreateDialog = true">创建权限规则</el-button>
      </div>
      <el-table :data="rules" stripe>
        <el-table-column prop="path" label="路径" />
        <el-table-column prop="role" label="角色" />
        <el-table-column prop="allowed_actions" label="允许操作" />
        <el-table-column prop="inherit" label="继承">
          <template #default="{ row }">{{ row.inherit ? '是' : '否' }}</template>
        </el-table-column>
        <el-table-column label="操作" width="120">
          <template #default="{ row }">
            <el-button size="small" type="danger" @click="deleteRule(row)">删除</el-button>
          </template>
        </el-table-column>
      </el-table>

      <el-dialog v-model="showCreateDialog" title="创建权限规则" width="400px">
        <el-form :model="createForm" label-width="80px">
          <el-form-item label="路径"><el-input v-model="createForm.path" placeholder="/documents" /></el-form-item>
          <el-form-item label="角色">
            <el-select v-model="createForm.role">
              <el-option label="只读" value="readonly" />
              <el-option label="读写" value="readwrite" />
              <el-option label="管理员" value="admin" />
            </el-select>
          </el-form-item>
          <el-form-item label="允许操作">
            <el-checkbox-group v-model="createForm.allowed_actions">
              <el-checkbox label="read" value="read" />
              <el-checkbox label="write" value="write" />
              <el-checkbox label="delete" value="delete" />
              <el-checkbox label="upload" value="upload" />
              <el-checkbox label="download" value="download" />
            </el-checkbox-group>
          </el-form-item>
          <el-form-item label="继承"><el-switch v-model="createForm.inherit" /></el-form-item>
        </el-form>
        <template #footer>
          <el-button @click="showCreateDialog = false">取消</el-button>
          <el-button type="primary" @click="doCreate">创建</el-button>
        </template>
      </el-dialog>
    </el-main>
  </el-container>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { adminApi } from '@/api/admin'
import { ElMessage, ElMessageBox } from 'element-plus'

const rules = ref<any[]>([])
const showCreateDialog = ref(false)
const createForm = ref({ path: '', role: 'readonly', allowed_actions: ['read'] as string[], inherit: true })

async function loadRules() {
  try {
    const res = await adminApi.listPermissions()
    rules.value = res.data.data || []
  } catch (e: any) { ElMessage.error('加载权限规则失败') }
}

async function doCreate() {
  try {
    await adminApi.createPermission(createForm.value)
    ElMessage.success('创建成功')
    showCreateDialog.value = false
    await loadRules()
  } catch (e: any) { ElMessage.error(e.message || '创建失败') }
}

async function deleteRule(row: any) {
  try {
    await ElMessageBox.confirm('确定删除此权限规则？', '确认', { type: 'warning' })
    await adminApi.deletePermission(row.id)
    ElMessage.success('删除成功')
    await loadRules()
  } catch { /* cancelled */ }
}

onMounted(() => { loadRules() })
</script>
