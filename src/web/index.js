import 'regenerator-runtime/runtime'
import {
  initContract,
  login,
  logout,
  isSignedIn,
  accountId,
  contract,
} from './wallet'

function behavior(name, fn) {
  document.querySelectorAll(`[data-behavior=${name}]`).forEach(fn)
}
const onclick = (fn) => (elem) => {
  elem.onclick = fn
}
const show = () => (elem) => {
  elem.style.display = 'block'
}
const hide = () => (elem) => {
  elem.style.display  = 'none'
}
const innerText = (text) => (elem) => {
  elem.innerText = text
}
const disabled = (status) => (elem) => {
  elem.disabled = status
}
const addClass = (name) => (elem) => {
  elem.classList.add(name);
}

behavior('login', onclick(login))
behavior('logout', onclick(logout))
behavior('action-burn', onclick(() => {
  behavior('action-burn', disabled(true))
  contract().avatar_burn().then(() => {
    behavior('action-burn', disabled(true))
    updateAvatar()
  }).catch(reason => {
    console.error(reason)
  })
}))
behavior('action-create', onclick(() => {
  behavior('action-create', disabled(true))
  contract().avatar_create().then(() => {
    behavior('action-create', disabled(true))
    updateAvatar()
  }).catch(reason => {
    console.error(reason)
  })
}))
behavior('logout', elem => {
  elem.onclick = logout
})

function signedInFlow() {
  behavior('signed-in-flow', show())
}

function signedOutFlow() {
  behavior('signed-out-flow', show())
  behavior('account-id', innerText(accountId()))
  return updateAvatar()
}

async function updateAvatar() {
  const avatar = await contract().avatar_of(accountId())
  behavior('avatar',elem => {
    elem.src = avatar
  })
  if ((await contract().avatar_exist(accountId()))) {
    behavior('action-burn', show())
    behavior('action-create', hide())
  } else {
    behavior('action-burn', hide())
    behavior('action-create', show())
    behavior('avatar', addClass('avatar_default'))
  }
}

initContract()
  .then(() => isSignedIn() ? signedOutFlow() : signedInFlow())
  .catch(console.error)
