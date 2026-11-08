import { ref } from 'vue'

export const currentUrl = ref(typeof location === 'undefined' ? '' : location.href)
