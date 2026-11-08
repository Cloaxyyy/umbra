import { defineConfig } from 'wxt'

const CHROME_KEY =
    'MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA87DP6Y8tfMzKTo5+Z5Gf+b1y2stXQwgSZtGmCpS5gwEmDcS98o0AD9/1RpuINo6IdFQk7Zbu9CgtKparzUl2tPlUz1HKYWo7lW+xKy6B68QFV4FKExpgkCK7rLbPmb2bQXGXbvMgnzyA8TtnEPWTCAmfSu7EaBJyw30QFnH/5sD9axT1Ko/PRiXRXAmThh0/s7GnCDtyViDiiawXdvqhP0ZpiR9bee47gv2fJKZq7xk6I6DRaROvJpbVPb0IZWFFDGIT7F/YZeW57zUUIdU+x/gzeo6VeC8QLtuBr3GmvEULAN1K8abt6CIyb99pvPpAIjBlTEzqS8oWQwIQP9CLdQIDAQAB'

export default defineConfig({
    modules: ['@wxt-dev/module-vue'],
    manifest: ({ browser }) => ({
        name: 'Umbra for Roblox',
        description: 'Custom themes, a better server browser, item values and one-click launches with the Umbra desktop app.',
        permissions: ['storage'],
        host_permissions: ['https://*.roblox.com/*', 'https://api.rolimons.com/*', 'http://127.0.0.1:27150/*'],
        action: { default_title: 'Umbra' },
        ...(browser === 'firefox'
            ? {
                  browser_specific_settings: {
                      gecko: {
                          id: 'umbra@cloaxy.github.io',
                          strict_min_version: '140.0',
                          data_collection_permissions: { required: ['none'] },
                      },
                  },
              }
            : { key: CHROME_KEY }),
    }),
})
