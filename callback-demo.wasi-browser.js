import {
  createOnMessage as __wasmCreateOnMessageForFsProxy,
  getDefaultContext as __emnapiGetDefaultContext,
  instantiateNapiModuleSync as __emnapiInstantiateNapiModuleSync,
  WASI as __WASI,
} from '@napi-rs/wasm-runtime'



const __wasi = new __WASI({
  version: 'preview1',
})

const __wasmUrl = new URL('./callback-demo.wasm32-wasi.wasm', import.meta.url).href
const __emnapiContext = __emnapiGetDefaultContext()


const __sharedMemory = new WebAssembly.Memory({
  initial: 4000,
  maximum: 65536,
  shared: true,
})

const __wasmFile = await fetch(__wasmUrl).then((res) => res.arrayBuffer())

const {
  instance: __napiInstance,
  module: __wasiModule,
  napiModule: __napiModule,
} = __emnapiInstantiateNapiModuleSync(__wasmFile, {
  context: __emnapiContext,
  asyncWorkPoolSize: 4,
  wasi: __wasi,
  onCreateWorker() {
    const worker = new Worker(new URL('./wasi-worker-browser.mjs', import.meta.url), {
      type: 'module',
    })

    return worker
  },
  overwriteImports(importObject) {
    importObject.env = {
      ...importObject.env,
      ...importObject.napi,
      ...importObject.emnapi,
      memory: __sharedMemory,
    }
    return importObject
  },
  beforeInit({ instance }) {
    for (const name of Object.keys(instance.exports)) {
      if (name.startsWith('__napi_register__')) {
        instance.exports[name]()
      }
    }
  },
})
export default __napiModule.exports
export const backgroundLogger = __napiModule.exports.backgroundLogger
export const calculateSalary = __napiModule.exports.calculateSalary
export const downloadFileWithProgress = __napiModule.exports.downloadFileWithProgress
export const fetchUserProfile = __napiModule.exports.fetchUserProfile
export const monitorSystemResources = __napiModule.exports.monitorSystemResources
export const optionalCallbackTest = __napiModule.exports.optionalCallbackTest
export const processEvents = __napiModule.exports.processEvents
export const processUserData = __napiModule.exports.processUserData
export const processWithFeedback = __napiModule.exports.processWithFeedback
export const scheduleNotification = __napiModule.exports.scheduleNotification
export const simpleCallbackTest = __napiModule.exports.simpleCallbackTest
export const startFileWatcher = __napiModule.exports.startFileWatcher
export const streamSensorData = __napiModule.exports.streamSensorData
