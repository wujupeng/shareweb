import api from './index'

export const uploadApi = {
  init(data: { file_name: string; target_path: string; total_size: number }) {
    return api.post('/upload/init', data)
  },
  uploadChunk(taskId: string, chunkIndex: number, data: ArrayBuffer) {
    return api.post(`/upload/chunk?task_id=${taskId}&chunk_index=${chunkIndex}`, data, {
      headers: { 'Content-Type': 'application/octet-stream' },
    })
  },
  complete(data: { task_id: string; file_name: string; target_path: string; total_chunks: number }) {
    return api.post('/upload/complete', data)
  },
  getStatus(taskId: string) {
    return api.get(`/upload/status?task_id=${taskId}`)
  },
}
