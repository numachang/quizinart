htmx.defineExtension('json-enc', {
  onEvent: (name, evt) => {
    if (name === 'htmx:configRequest') {
      evt.detail.headers['Content-Type'] = 'application/json'
    }
  },

  encodeParameters: (xhr, parameters, elt) => {
    xhr.overrideMimeType('text/json')
    return JSON.stringify(parameters)
  },
})
