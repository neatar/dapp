import {Contract} from 'near-api-js'

export class Neatar {
  /**
   * @type {Contract}
   */
  contract

  constructor(account, contractId) {
    this.contract = new Contract(account, contractId, {
      viewMethods: [
        'avatar_of',
        'nft_metadata',
      ],
      changeMethods: [
        'new',
        'avatar_create',
        'avatar_create_for',
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
    return this.contract.avatar_create()
  }

  /**
   * @param {string} account_id
   * @returns {Promise<string>}
   */
  avatar_create_for(account_id) {
    return this.contract.avatar_create_for({account_id})
  }

  /**
   * @returns {Promise<Object>}
   */
  nft_metadata() {
    return this.contract.nft_metadata()
  }
}
