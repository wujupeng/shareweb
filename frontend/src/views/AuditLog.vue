<template>
  <el-container style="height: 100vh">
    <el-header style="display: flex; align-items: center; justify-content: space-between; border-bottom: 1px solid #e4e7ed; padding: 0 16px">
      <h2 style="margin: 0">审计日志</h2>
      <div>
        <el-button @click="$router.push('/')">返回文件浏览</el-button>
      </div>
    </el-header>
    <el-main>
      <div style="margin-bottom: 16px; display: flex; gap: 12px">
        <el-input v-model="filters.operator" placeholder="操作者" style="width: 150px" clearable />
        <el-select v-model="filters.action_type" placeholder="操作类型" style="width: 150px" clearable>
          <el-option label="登录" value="login" />
          <el-option label="登出" value="logout" />
          <el-option label="上传" value="upload" />
          <el-option label="下载" value="download" />
          <el-option label="删除" value="delete" />
          <el-option label="重命名" value="rename" />
          <el-option label="移动" value="move" />
          <el-option label="复制" value="copy" />
        </el-select>
        <el-button type="primary" @click="loadLogs">查询</el-button>
      </div>
      <el-table :data="logs" stripe>
        <el-table-column prop="operator" label="操作者" width="120" />
        <el-table-column prop="action_type" label="操作类型" width="120" />
        <el-table-column prop="target_path" label="目标路径" />
        <el-table-column prop="result" label="结果" width="80">
          <template #default="{ row }">
            <el-tag :type="row.result === 'success' ? 'success' : 'danger'">{{ row.result }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="source_ip" label="来源IP" width="140" />
        <el-table-column prop="action_time" label="操作时间" width="180" />
      </el-table>
    </el-main>
  </el-container>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { adminApi } from '@/api/admin'
import { ElMessage } from 'element-plus'

const logs = ref<any[]>([])
const filters = reactive({ operator: '', action_type: '' })

async function loadLogs() {
  try {
    const params: any = {}
    if (filters.operator) params.operator = filters.operator
    if (filters.action_type) params.action_type = filters.action_type
    const res = await adminApi.listAuditLogs(params)
    logs.value = res.data.data.items || []
  } catch (e: any) { ElMessage.error('加载审计日志失败') }
}

onMounted(() => { loadLogs() })
</script>
