import 'regenerator-runtime/runtime'
import { initContract, login, logout, isSignedIn, accountId } from './wallet'

let avatar

document.querySelectorAll('[data-behavior=login]').forEach(elem => {
  elem.onclick = login
})
document.querySelectorAll('[data-behavior=logout]').forEach(elem => {
  elem.onclick = logout
})

function signedInFlow() {
  document.querySelectorAll('[data-behavior=signed-in-flow]').forEach(elem => {
    elem.style = 'block'
  })
}

function signedOutFlow() {
  document.querySelectorAll('[data-behavior=signed-out-flow]').forEach(elem => {
    elem.style = 'block'
  })
  document.querySelectorAll('[data-behavior=account-id]').forEach(elem => {
    elem.innerText = accountId()
  })
  return fetchAvatar()
}

async function fetchAvatar() {
  console.log(isSignedIn())
  console.log(accountId())
  avatar = await contract.avatar_of(accountId())
  document.querySelectorAll('[data-behavior=avatar]').forEach(elem => {
    elem.src = avatar
  })
}

initContract()
  .then(() => isSignedIn() ? signedOutFlow() : signedInFlow())
  .catch(console.error)
