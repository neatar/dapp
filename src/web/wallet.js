import {connect, keyStores, WalletConnection} from 'near-api-js'
import {config} from './config'
import {Neatar} from '../contract/neatar/neatar'

const nearConfig = config('testnet')

// Initialize contract & set global variables
export async function initContract() {
  // Initialize connection to the NEAR
  const near = await connect(Object.assign({
    deps: {
      keyStore: new keyStores.BrowserLocalStorageKeyStore(),
    },
  }, nearConfig))
  // Initializing Wallet based Account
  window.walletConnection = new WalletConnection(near, nearConfig.networkId)
  // Getting the Account ID. If still unauthorized, it's just empty string
  window.accountId = window.walletConnection.getAccountId()
  // Initializing our contract APIs by contract name and configuration
  window.contract = new Neatar(window.walletConnection.account(), nearConfig.contractName)
}

export function logout() {
  window.walletConnection.signOut()
  window.location.replace(window.location.origin + window.location.pathname) // reload page
}

export function login() {
  window.walletConnection.requestSignIn(nearConfig.contractName)
}

export function isSignedIn() {
  return window.walletConnection.isSignedIn()
}

export function accountId() {
  return window.accountId
}
