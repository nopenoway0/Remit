class RemitUtilities {
    static extract_elements(ids) {
        let data = {};
        ids.forEach((id) => {
            data[id] = document.getElementById(id).value;
        })
        return data;
    }

    static filled_string(s) {
        return (s != undefined && s.length > 0);
    }
}

export default RemitUtilities;