
window.onload = function(e){
	import("./core/pkg/core.js").then(module => {
		client = new module.Core()
		client.start()
	})
}
