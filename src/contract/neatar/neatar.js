import {Contract} from 'near-api-js'

const AVATAR_CREATE_STORAGE_COST = '50000000000000000000000' // 0.05 NEAR
const AVATAR_CREATE_PREPAID_GAS = '300000000000000' // 300 TGas

export class Neatar {
  /**
   * @type {Contract}
   */
  contract

  constructor(account, contractId) {
    this.contract = new Contract(account, contractId, {
      viewMethods: [
        'avatar_of',
        'avatar_exist',
        'nft_tokens_for_owner',
        'nft_metadata',
      ],
      changeMethods: [
        'new',
        'avatar_create',
        'avatar_create_for',
        'avatar_burn_for',
        'avatar_burn',
        'ft_burn',
      ],
    })
  }

  /**
   * @param {string} account_id
   * @returns {Promise<string>}
   */
  avatar_of(account_id) {
    return this.contract.avatar_of({account_id})
  }

  /**
   * @returns {Promise<string>}
   */
  avatar_create() {
    return this.contract.avatar_create({}, AVATAR_CREATE_PREPAID_GAS, AVATAR_CREATE_STORAGE_COST)
  }

  /**
   * @param {string} account_id
   * @returns {Promise<string>}
   */
  avatar_create_for(account_id) {
    return this.contract.avatar_create_for({account_id})
  }

  /**
   * @returns {Promise<void>}
   */
  avatar_burn() {
    return this.contract.avatar_burn({}, AVATAR_CREATE_PREPAID_GAS)
  }

  /**
   * @returns {Promise<Object>}
   */
  nft_metadata() {
    return this.contract.nft_metadata()
  }

  /**
   * @param {string} account_id
   * @returns {Promise<bolean>}
   */
  avatar_exist(account_id) {
    return this.contract.avatar_exist({account_id})
  }

  /**
   * @param {string} account_id
   * @param {string} from_index
   * @param {number} limit
   * @returns {Promise<Object[]>}
   */
  nft_tokens_for_owner(account_id, from_index = "0", limit = 50) {
    return this.contract.nft_tokens_for_owner({
      account_id,
      from_index,
      limit,
    })
  }
}
