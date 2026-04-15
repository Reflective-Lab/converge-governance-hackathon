// Borrowed from Wolfgang — fun verbs to show while working
export const SPINNER_VERBS = [
	'Accomplishing', 'Actioning', 'Actualizing', 'Architecting', 'Baking',
	'Beaming', "Beboppin'", 'Befuddling', 'Billowing', 'Blanching',
	'Bloviating', 'Boogieing', 'Boondoggling', 'Booping', 'Bootstrapping',
	'Brewing', 'Bunning', 'Burrowing', 'Calculating', 'Canoodling',
	'Caramelizing', 'Cascading', 'Catapulting', 'Cerebrating', 'Channeling',
	'Choreographing', 'Churning', 'Clauding', 'Coalescing', 'Cogitating',
	'Combobulating', 'Composing', 'Computing', 'Concocting', 'Considering',
	'Contemplating', 'Cooking', 'Crafting', 'Creating', 'Crunching',
	'Crystallizing', 'Cultivating', 'Deciphering', 'Deliberating', 'Determining',
	'Dilly-dallying', 'Discombobulating', 'Doodling', 'Drizzling', 'Ebbing',
	'Elucidating', 'Embellishing', 'Enchanting', 'Envisioning', 'Fermenting',
	'Fiddle-faddling', 'Finagling', 'Flambeing', 'Flibbertigibbeting', 'Flowing',
	'Flummoxing', 'Fluttering', 'Forging', 'Forming', 'Frolicking', 'Frosting',
	'Gallivanting', 'Galloping', 'Garnishing', 'Generating', 'Gesticulating',
	'Germinating', 'Grooving', 'Gusting', 'Harmonizing', 'Hashing', 'Hatching',
	'Herding', 'Hullaballooing', 'Hyperspacing', 'Ideating', 'Imagining',
	'Improvising', 'Incubating', 'Inferring', 'Infusing', 'Ionizing',
	'Jitterbugging', 'Julienning', 'Kneading', 'Leavening', 'Levitating',
	'Lollygagging', 'Manifesting', 'Marinating', 'Meandering', 'Metamorphosing',
	'Misting', 'Moonwalking', 'Moseying', 'Mulling', 'Mustering', 'Musing',
	'Nebulizing', 'Nesting', 'Noodling', 'Nucleating', 'Orbiting',
	'Orchestrating', 'Osmosing', 'Perambulating', 'Percolating', 'Perusing',
	'Philosophising', 'Photosynthesizing', 'Pollinating', 'Pondering',
	'Pontificating', 'Precipitating', 'Prestidigitating', 'Processing',
	'Proofing', 'Propagating', 'Puttering', 'Puzzling', 'Quantumizing',
	'Razzle-dazzling', 'Razzmatazzing', 'Recombobulating', 'Reticulating',
	'Ruminating', 'Sauteing', 'Scampering', 'Schlepping', 'Scurrying',
	'Seasoning', 'Shenaniganing', 'Shimmying', 'Simmering', 'Skedaddling',
	'Sketching', 'Smooshing', 'Sock-hopping', 'Spelunking', 'Spinning',
	'Sprouting', 'Stewing', 'Sublimating', 'Swirling', 'Swooping',
	'Symbioting', 'Synthesizing', 'Tempering', 'Thinking', 'Thundering',
	'Tinkering', 'Tomfoolering', 'Topsy-turvying', 'Transfiguring',
	'Transmuting', 'Twisting', 'Undulating', 'Unfurling', 'Unravelling',
	'Vibing', 'Waddling', 'Wandering', 'Warping', 'Whatchamacalliting',
	'Whirlpooling', 'Whirring', 'Whisking', 'Wibbling', 'Wrangling',
	'Zesting', 'Zigzagging',
]

export function randomVerb(): string {
	return SPINNER_VERBS[Math.floor(Math.random() * SPINNER_VERBS.length)] ?? 'Thinking'
}

export function seededVerb(seed: string, tick = 0): string {
	let hash = 0
	for (let i = 0; i < seed.length; i++) {
		hash = Math.imul(31, hash) + seed.charCodeAt(i)
		hash >>>= 0
	}
	const index = (hash + tick) % SPINNER_VERBS.length
	return SPINNER_VERBS[index] ?? 'Thinking'
}
