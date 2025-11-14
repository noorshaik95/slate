use prometheus::{
    register_counter_vec, register_gauge, register_histogram_vec, CounterVec, Gauge, HistogramVec,
    Registry, TextEncoder, Encoder,
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
        let uploads_total = register_counter_vec!(
            "cms_uploads_total",
            "Total number of uploads by status",
            &["status"]
        )?;

        let upload_duration = register_histogram_vec!(
            "cms_upload_duration_seconds",
            "Upload duration in seconds",
            &["content_type"]
        )?;

        let upload_size = register_histogram_vec!(
            "cms_upload_size_bytes",
            "Upload file size in bytes",
            &["content_type"]
        )?;

        let active_upload_sessions = register_gauge!(
            "cms_active_upload_sessions",
            "Number of active upload sessions"
        )?;

        // Streaming metrics
        let video_streams_total = register_counter_vec!(
            "cms_video_streams_total",
            "Total number of video stream requests",
            &["quality"]
        )?;

        let video_playback_duration = register_histogram_vec!(
            "cms_video_playback_duration_seconds",
            "Video playback duration in seconds",
            &["video_id"]
        )?;

        let video_completion_rate = register_gauge!(
            "cms_video_completion_rate",
            "Video completion rate percentage"
        )?;

        // Search metrics
        let search_queries_total = register_counter_vec!(
            "cms_search_queries_total",
            "Total number of search queries",
            &["status"]
        )?;

        let search_duration = register_histogram_vec!(
            "cms_search_duration_seconds",
            "Search query duration in seconds",
            &["status"]
        )?;

        let search_results_count = register_histogram_vec!(
            "cms_search_results_count",
            "Number of search results returned",
            &["query_type"]
        )?;

        // Transcoding metrics
        let transcoding_jobs_total = register_counter_vec!(
            "cms_transcoding_jobs_total",
            "Total number of transcoding jobs",
            &["status"]
        )?;

        let transcoding_duration = register_histogram_vec!(
            "cms_transcoding_duration_seconds",
            "Transcoding job duration in seconds",
            &["format"]
        )?;

        let transcoding_queue_size = register_gauge!(
            "cms_transcoding_queue_size",
            "Number of pending transcoding jobs"
        )?;

        // Database metrics
        let db_connections_active = register_gauge!(
            "cms_db_connections_active",
            "Number of active database connections"
        )?;

        let db_query_duration = register_histogram_vec!(
            "cms_db_query_duration_seconds",
            "Database query duration in seconds",
            &["operation"]
        )?;

        let db_errors_total = register_counter_vec!(
            "cms_db_errors_total",
            "Total number of database errors",
            &["error_type"]
        )?;

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
