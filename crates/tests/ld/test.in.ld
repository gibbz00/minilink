SECTIONS {
	.test : {
		{% if contains(cfg.feature, "some_feature") %}
		__feature = .;
		{% endif %}
		{% if cfg.test %}
		__test = .;
		{% endif %}
	}
}
