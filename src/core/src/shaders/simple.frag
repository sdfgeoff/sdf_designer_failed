precision mediump float;
varying vec4 screen_pos;

void main() {
	vec4 col = screen_pos * 0.5 + 0.5;
	col.a = 1.0;
	gl_FragColor = col;
}
