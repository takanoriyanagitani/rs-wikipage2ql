#!/bin/sh

export ENV_ADDR_PORT=127.0.0.1:8109

nsquery='query ($ofst: Int!, $sz: Int!, $ns: String!, $hr: Boolean!){
	pages(offset: $ofst, size: $sz, filter: { namespace: $ns, hasRedirect: $hr }){
		title
		namespace
		id
		restrictions
		redirect { title }
		revision {
			id
			parentId
			timestamp
			comment
			origin
			model
			format
			sha1
		}
	}
}'

jq \
	-c \
	-n \
	--arg query "${nsquery}" \
	--argjson args '{
		"ofst": 569,
		"sz": 704019,
		"ns": "0",
		"hr": false
	}' \
	'{
		query: $query,
		variables: $args,
	}' |
	curl \
		--silent \
		--show-error \
		--fail \
		--location \
		--data @- \
		"${ENV_ADDR_PORT}" |
	jq -c '.data.pages.[]' |
	tail -2 |
	dasel --read=json --write=yaml |
	bat --language=yaml
