use prometheus::{
    CounterVec, Encoder, Gauge, HistogramOpts, HistogramVec, Opts, Registry, TextEncoder,
};
use std::sync::Arc;

/// Metrics collector for the Content Management Service
#[derive(Clone)]
pub struct Metrics {
    // Upload metrics
    pub uploads_total: CounterVec,
    pub upload_duration: HistogramVec,
    pub upload_size: HistogramVec,
    pub active_upload_sessions: Gauge,

    // Streaming metrics
    pub video_streams_total: CounterVec,
    pub video_playback_duration: HistogramVec,
    pub video_completion_rate: Gauge,

    // Search metrics
    pub search_queries_total: CounterVec,
    pub search_duration: HistogramVec,
    pub search_results_count: HistogramVec,

    // Transcoding metrics
    pub transcoding_jobs_total: CounterVec,
    pub transcoding_duration: HistogramVec,
    pub transcoding_queue_size: Gauge,

    // Database metrics
    pub db_connections_active: Gauge,
    pub db_query_duration: HistogramVec,
    pub db_errors_total: CounterVec,

    registry: Arc<Registry>,
}

impl Metrics {
    /// Create a new metrics collector
    pub fn new() -> anyhow::Result<Self> {
        let registry = Registry::new();

        // Upload metrics
        let uploads_total = CounterVec::new(
            Opts::new("cms_uploads_total", "Total number of uploads by status"),
            &["status"],
        )?;
        registry.register(Box::new(uploads_total.clone()))?;

        let upload_duration = HistogramVec::new(
            HistogramOpts::new("cms_upload_duration_seconds", "Upload duration in seconds"),
            &["content_type"],
        )?;
        registry.register(Box::new(upload_duration.clone()))?;

        let upload_size = HistogramVec::new(
            HistogramOpts::new("cms_upload_size_bytes", "Upload file size in bytes").buckets(vec![
                1_000_000.0,   // 1MB
                5_000_000.0,   // 5MB
                10_000_000.0,  // 10MB
                50_000_000.0,  // 50MB
                100_000_000.0, // 100MB
                250_000_000.0, // 250MB
                500_000_000.0, // 500MB
            ]),
            &["content_type"],
        )?;
        registry.register(Box::new(upload_size.clone()))?;

        let active_upload_sessions = Gauge::new(
            "cms_active_upload_sessions",
            "Number of active upload sessions",
        )?;
        registry.register(Box::new(active_upload_sessions.clone()))?;

        // Streaming metrics
        let video_streams_total = CounterVec::new(
            Opts::new(
                "cms_video_streams_total",
                "Total number of video stream requests",
            ),
            &["quality"],
        )?;
        registry.register(Box::new(video_streams_total.clone()))?;

        let video_playback_duration = HistogramVec::new(
            HistogramOpts::new(
                "cms_video_playback_duration_seconds",
                "Video playback duration in seconds",
            )
            .buckets(vec![10.0, 30.0, 60.0, 300.0, 600.0, 1800.0, 3600.0]),
            &["resource_id"],
        )?;
        registry.register(Box::new(video_playback_duration.clone()))?;

        let video_completion_rate = Gauge::new(
            "cms_video_completion_rate",
            "Video completion rate percentage",
        )?;
        registry.register(Box::new(video_completion_rate.clone()))?;

        // Search metrics
        let search_queries_total = CounterVec::new(
            Opts::new("cms_search_queries_total", "Total number of search queries"),
            &["status"],
        )?;
        registry.register(Box::new(search_queries_total.clone()))?;

        let search_duration = HistogramVec::new(
            HistogramOpts::new(
                "cms_search_duration_seconds",
                "Search query duration in seconds",
            )
            .buckets(vec![0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.0]),
            &["status"],
        )?;
        registry.register(Box::new(search_duration.clone()))?;

        let search_results_count = HistogramVec::new(
            HistogramOpts::new(
                "cms_search_results_count",
                "Number of search results returned",
            )
            .buckets(vec![0.0, 1.0, 5.0, 10.0, 25.0, 50.0]),
            &["query_type"],
        )?;
        registry.register(Box::new(search_results_count.clone()))?;

        // Transcoding metrics
        let transcoding_jobs_total = CounterVec::new(
            Opts::new(
                "cms_transcoding_jobs_total",
                "Total number of transcoding jobs",
            ),
            &["status"],
        )?;
        registry.register(Box::new(transcoding_jobs_total.clone()))?;

        let transcoding_duration = HistogramVec::new(
            HistogramOpts::new(
                "cms_transcoding_duration_seconds",
                "Transcoding job duration in seconds",
            )
            .buckets(vec![10.0, 30.0, 60.0, 120.0, 300.0, 600.0, 1800.0]),
            &["format"],
        )?;
        registry.register(Box::new(transcoding_duration.clone()))?;

        let transcoding_queue_size = Gauge::new(
            "cms_transcoding_queue_size",
            "Number of pending transcoding jobs",
        )?;
        registry.register(Box::new(transcoding_queue_size.clone()))?;

        // Database metrics
        let db_connections_active = Gauge::new(
            "cms_db_connections_active",
            "Number of active database connections",
        )?;
        registry.register(Box::new(db_connections_active.clone()))?;

        let db_query_duration = HistogramVec::new(
            HistogramOpts::new(
                "cms_db_query_duration_seconds",
                "Database query duration in seconds",
            )
            .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0]),
            &["operation"],
        )?;
        registry.register(Box::new(db_query_duration.clone()))?;

        let db_errors_total = CounterVec::new(
            Opts::new("cms_db_errors_total", "Total number of database errors"),
            &["error_type"],
        )?;
        registry.register(Box::new(db_errors_total.clone()))?;

        Ok(Self {
            uploads_total,
            upload_duration,
            upload_size,
            active_upload_sessions,
            video_streams_total,
            video_playback_duration,
            video_completion_rate,
            search_queries_total,
            search_duration,
            search_results_count,
            transcoding_jobs_total,
            transcoding_duration,
            transcoding_queue_size,
            db_connections_active,
            db_query_duration,
            db_errors_total,
            registry: Arc::new(registry),
        })
    }

    /// Encode metrics in Prometheus text format
    pub fn encode(&self) -> anyhow::Result<String> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)?;
        Ok(String::from_utf8(buffer)?)
    }

    /// Get the registry for this metrics collector
    pub fn registry(&self) -> &Registry {
        &self.registry
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new().expect("Failed to create metrics")
    }
}

/// Initialize metrics collector
pub fn init_metrics() -> anyhow::Result<Metrics> {
    Metrics::new()
}
