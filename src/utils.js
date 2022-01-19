/**
 * Contains utility functions that are used through the application
 */
class RemitUtilities {
    /**
     * Extract data from the DOM using a list of strings and then return map id->value
     * @param {string[]} ids A list of ids to extract data from 
     * @returns {Object<string, string>} A map of element_id and their values after extracting them from the DOM 
     */
    static extract_elements(ids) {
        let data = {};
        ids.forEach((id) => {
            data[id] = document.getElementById(id).value;
        })
        return data;
    }

    /**
     * Tests whether the incoming string is undefined and > 0
     * @param {string} s The string to test
     * @returns {bool} False if the string is undefined or <= 0. Otherwise, return true
     */
    static filled_string(s) {
        return (s != undefined && s.length > 0);
    }

    /**
     * Takes a string and converts it to a form that's usable as an id in the DOM
     * @param {string} str 
     * @returns {string} The string transformed to lower case and spaces replaced with _
     */
    static string_to_key(str) {
        return str.toLowerCase().replace(" ", "_");
    }
}


export default RemitUtilities;