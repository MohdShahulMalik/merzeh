DEFINE EVENT cleanup_expired_session ON TABLE sessions WHEN $before != NONE THEN (
	DELETE sessions WHERE expires_at <= time::now()
);
