<template>
  <el-container style="height: 100vh">
    <el-header style="display: flex; align-items: center; justify-content: space-between; border-bottom: 1px solid #e4e7ed; padding: 0 16px">
      <h2 style="margin: 0">用户管理</h2>
      <div>
        <el-button @click="$router.push('/')">返回文件浏览</el-button>
        <el-button @click="authStore.logout(); $router.push('/login')">退出</el-button>
      </div>
    </el-header>
    <el-main>
      <div style="margin-bottom: 16px">
        <el-button type="primary" @click="showCreateDialog = true">创建用户</el-button>
      </div>
      <el-table :data="users" stripe>
        <el-table-column prop="username" label="用户名" />
        <el-table-column prop="role" label="角色" />
        <el-table-column prop="status" label="状态" />
        <el-table-column prop="created_at" label="创建时间" />
        <el-table-column label="操作" width="250">
          <template #default="{ row }">
            <el-button size="small" @click="editUser(row)">编辑</el-button>
            <el-button size="small" type="danger" @click="deleteUser(row)" :disabled="row.username === 'admin'">删除</el-button>
          </template>
        </el-table-column>
      </el-table>

      <el-dialog v-model="showCreateDialog" title="创建用户" width="400px">
        <el-form :model="createForm" label-width="80px">
          <el-form-item label="用户名"><el-input v-model="createForm.username" /></el-form-item>
          <el-form-item label="密码"><el-input v-model="createForm.password" type="password" show-password /></el-form-item>
          <el-form-item label="角色">
            <el-select v-model="createForm.role">
              <el-option label="管理员" value="admin" />
              <el-option label="读写" value="readwrite" />
              <el-option label="只读" value="readonly" />
            </el-select>
          </el-form-item>
        </el-form>
        <template #footer>
          <el-button @click="showCreateDialog = false">取消</el-button>
          <el-button type="primary" @click="doCreateUser">创建</el-button>
        </template>
      </el-dialog>

      <el-dialog v-model="showEditDialog" title="编辑用户" width="400px">
        <el-form :model="editForm" label-width="80px">
          <el-form-item label="角色">
            <el-select v-model="editForm.role">
              <el-option label="管理员" value="admin" />
              <el-option label="读写" value="readwrite" />
              <el-option label="只读" value="readonly" />
            </el-select>
          </el-form-item>
          <el-form-item label="状态">
            <el-select v-model="editForm.status">
              <el-option label="活跃" value="active" />
              <el-option label="禁用" value="disabled" />
            </el-select>
          </el-form-item>
          <el-form-item label="新密码"><el-input v-model="editForm.password" type="password" show-password placeholder="留空不修改" /></el-form-item>
        </el-form>
        <template #footer>
          <el-button @click="showEditDialog = false">取消</el-button>
          <el-button type="primary" @click="doUpdateUser">保存</el-button>
        </template>
      </el-dialog>
    </el-main>
  </el-container>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useAuthStore } from '@/stores/auth'
import { adminApi } from '@/api/admin'
import { ElMessage, ElMessageBox } from 'element-plus'

const authStore = useAuthStore()
const users = ref<any[]>([])
const showCreateDialog = ref(false)
const showEditDialog = ref(false)
const createForm = ref({ username: '', password: '', role: 'readonly' })
const editForm = ref({ username: '', role: 'readonly', status: 'active', password: '' })

async function loadUsers() {
  try {
    const res = await adminApi.listUsers()
    users.value = res.data.data.items || []
  } catch (e: any) { ElMessage.error('加载用户列表失败') }
}

async function doCreateUser() {
  try {
    await adminApi.createUser(createForm.value)
    ElMessage.success('用户创建成功')
    showCreateDialog.value = false
    createForm.value = { username: '', password: '', role: 'readonly' }
    await loadUsers()
  } catch (e: any) { ElMessage.error(e.message || '创建失败') }
}

function editUser(row: any) {
  editForm.value = { username: row.username, role: row.role, status: row.status, password: '' }
  showEditDialog.value = true
}

async function doUpdateUser() {
  try {
    const data: any = { role: editForm.value.role, status: editForm.value.status }
    if (editForm.value.password) data.password = editForm.value.password
    await adminApi.updateUser(editForm.value.username, data)
    ElMessage.success('更新成功')
    showEditDialog.value = false
    await loadUsers()
  } catch (e: any) { ElMessage.error(e.message || '更新失败') }
}

async function deleteUser(row: any) {
  try {
    await ElMessageBox.confirm(`确定删除用户 ${row.username} 吗？`, '确认', { type: 'warning' })
    await adminApi.deleteUser(row.username)
    ElMessage.success('删除成功')
    await loadUsers()
  } catch { /* cancelled */ }
}

onMounted(() => { loadUsers() })
</script>
