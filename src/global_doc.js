/**
 * A function that takes no arguments and returns no values
 * @callback NoArgNoReturnCallback
 */

/**
 * @typedef {Object} FormField
 * @property {*} value Value of the field
 */

/**
 * Process any amount of args and return a value
 * @callback GenericCallback
 * @param {...*} args
 * @return {*}
 */

/**
 * Standard RemitConfiguration Object
 * @typedef {Object} RemitConfiguration
 * @property {string} username username
 * @property {string} name configuration name
 * @property {string} host configuration host
 * @property {number} port ssh port on server
 * @property {string} password password
 */

/**
 * Basic file information as collected from the Rust backend list_current_directory function
 * @typedef {Object} Navigator~ListFile
 * @property {string} name
 * @property {FileType} type
 * @property {size} number
 */

/**
 * @typedef {Object} TextFieldRecipe
 * @property {string} label
 * @property {string} type
 * @property {bool} error
 * @property {string} error_text
 */

/**
 * name->RemitConfiguration Map
 * @typedef {Object<string, RemitConfiguration>} RemitConfigurationDict
 */