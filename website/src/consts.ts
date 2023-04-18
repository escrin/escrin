export const SITE = {
	title: 'Escrin',
	description:
		'Escrin is a secure decentralized computing network that enables private computation on multi-source private data.',
	defaultLanguage: 'en-us',
} as const;

export const OPEN_GRAPH = {
	image: {
		src: '',
		alt: '',
	},
	twitter: '',
};

export const KNOWN_LANGUAGES = {
	English: 'en',
} as const;
export const KNOWN_LANGUAGE_CODES = Object.values(KNOWN_LANGUAGES);

export const GITHUB_EDIT_URL = `https://github.com/escrin/escrin/tree/main/website`;

export const COMMUNITY_INVITE_URL = 'https://discord.gg/KpNYB2F42a';

// See "Algolia" section of the README for more information.
export const ALGOLIA = {
	indexName: 'escrin',
	appId: 'ZNRK6V99NY',
	apiKey: 'ab132c7d3e214170645c7e45a41094dd',
};

export type Sidebar = Record<
	(typeof KNOWN_LANGUAGE_CODES)[number],
	Record<string, { text: string; link: string }[]>
>;
export const SIDEBAR: Sidebar = {
	en: {
		'Getting Started': [
			{ text: 'What is Escrin?', link: 'en/getting-started/introduction' },
			{ text: 'My First Task', link: 'en/getting-started/my-first-task' },
			{ text: 'My Second Task', link: 'en/coming-soon' },
			{ text: 'Next Steps', link: 'en/coming-soon' },
		],
		'How-To Guides': [
			{ text: 'DAO Off-Chain Agent', link: 'en/coming-soon' },
			{ text: 'Private Data Marketplace', link: 'en/coming-soon' },
			{ text: 'Secure Game Server', link: 'en/coming-soon' },
			{ text: 'DeFi-CeFi Bridge', link: 'en/coming-soon' },
			{ text: 'Identity & Selective Disclosure', link: 'en/coming-soon' },
			{ text: 'Personalized AI', link: 'en/coming-soon' },
			{ text: 'Sealed MEV Auctions', link: 'en/coming-soon' },
			// { text: 'DAO Off-Chain Agent', link: 'en/how-to/off-chain-agent' },
			// { text: 'Private Data Marketplace', link: 'en/how-to/data-marketplace' },
			// { text: 'Secure Game Server', link: 'en/how-to/secure-game-instances' },
			// { text: 'DeFi-CeFi Bridge', link: 'en/how-to/decefi-bridge' },
			// { text: 'Selective Identity', link: 'en/how-to/selective-identity' },
			// { text: 'Personalized AI', link: 'en/how-to/personalized-ai' },
		],
		'Architecture': [
			{ text: 'Overview', link: 'en/coming-soon' },
			// { text: 'Overview', link: 'en/architecture/overview' },
		],
	},
};
