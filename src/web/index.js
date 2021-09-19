import 'regenerator-runtime/runtime'
import { initContract, login, logout, isSignedIn, accountId } from './wallet'

function behavior(name, fn) {
  document.querySelectorAll(`[data-behavior=${name}]`).forEach(fn)
}
const onclick = (fn) => (elem) => {
  elem.onclick = fn
}
const show = () => (elem) => {
  elem.style = 'block'
}
const innerText = (text) => (elem) => {
  elem.innerText = text
}

behavior('login', onclick(login))
behavior('logout', onclick(logout))
behavior('logout', elem => {
  elem.onclick = logout
})

function signedInFlow() {
  behavior('signed-in-flow', show())
}

function signedOutFlow() {
  behavior('signed-out-flow', show())
  behavior('account-id', innerText(accountId()))
  return fetchAvatar()
}

async function fetchAvatar() {
  const avatar = await contract.avatar_of(accountId())
  behavior('avatar',elem => {
    elem.src = avatar
  })
}

initContract()
  .then(() => isSignedIn() ? signedOutFlow() : signedInFlow())
  .catch(console.error)
