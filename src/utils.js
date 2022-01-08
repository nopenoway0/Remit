class RemitUtilities {
    static extract_elements(ids) {
        let data = {};
        ids.forEach((id) => {
            data[id] = document.getElementById(id).value;
        })
        return data;
    }
}

export default RemitUtilities;