use crate::{consensus::core::network::PyNetworkId, rpc::encoding::PyEncoding};
use kaspa_wrpc_client::Resolver;
use pyo3::{exceptions::PyException, prelude::*};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use std::sync::Arc;

/// A resolver for discovering Kaspa RPC node endpoints.
///
/// Resolvers help clients find available nodes on a network by querying
/// a list of known resolver URLs. Useful for automatic node discovery
/// and load balancing.
#[gen_stub_pyclass]
#[pyclass(name = "Resolver")]
#[derive(Debug, Clone)]
pub struct PyResolver(Resolver);

impl PyResolver {
    pub fn new(resolver: Resolver) -> Self {
        Self(resolver)
    }

    pub fn inner(&self) -> Resolver {
        self.0.clone()
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl PyResolver {
    /// Create a new resolver.
    ///
    /// Args:
    ///     urls: Optional list of resolver URLs. Uses defaults if not provided.
    ///     tls: Use TLS connections (default: False).
    ///
    /// Returns:
    ///     Resolver: A new Resolver instance.
    #[new]
    #[pyo3(signature = (urls=None, tls=None))]
    pub fn ctor(urls: Option<Vec<String>>, tls: Option<bool>) -> PyResult<Self> {
        let tls = tls.unwrap_or(false);
        if let Some(urls) = urls {
            Ok(Self(Resolver::new(
                Some(urls.into_iter().map(Arc::new).collect::<Vec<_>>()),
                tls,
            )))
        } else {
            Ok(Self(Resolver::default()))
        }
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl PyResolver {
    /// Get the list of resolver URLs.
    ///
    /// Returns:
    ///     list[str]: The resolver URL list.
    fn urls(&self) -> Vec<String> {
        self.0
            .urls()
            .unwrap_or_default()
            .into_iter()
            .map(|url| (*url).clone())
            .collect::<Vec<_>>()
    }

    /// Get a node descriptor from the resolver (async).
    ///
    /// Args:
    ///     encoding: RPC encoding - either a string ("borsh" or "json") or an Encoding enum variant.
    ///     network_id: The network to find a node for.
    ///
    /// Returns:
    ///     dict: Node descriptor with connection details.
    ///
    /// Raises:
    ///     Exception: If no node is available or resolution fails.
    #[gen_stub(override_return_type(type_repr = "dict"))]
    fn get_node<'py>(
        &self,
        py: Python<'py>,
        #[gen_stub(override_type(type_repr = "str | Encoding"))] encoding: PyEncoding,
        network_id: PyNetworkId,
    ) -> PyResult<Bound<'py, PyAny>> {
        let resolver = self.0.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let node = resolver
                .get_node(encoding.into(), network_id.into())
                .await
                .map_err(|err| PyException::new_err(err.to_string()))?;
            Python::attach(|py| Ok(serde_pyobject::to_pyobject(py, &node)?.unbind()))
        })
    }

    /// Get a node URL from the resolver (async).
    ///
    /// Args:
    ///     encoding: RPC encoding - either a string ("borsh" or "json") or an Encoding enum variant.
    ///     network_id: The network to find a node for.
    ///
    /// Returns:
    ///     str: The node WebSocket URL.
    ///
    /// Raises:
    ///     Exception: If no node is available or resolution fails.
    #[gen_stub(override_return_type(type_repr = "str"))]
    fn get_url<'py>(
        &self,
        py: Python<'py>,
        #[gen_stub(override_type(type_repr = "str | Encoding"))] encoding: PyEncoding,
        network_id: PyNetworkId,
    ) -> PyResult<Bound<'py, PyAny>> {
        let resolver = self.0.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let url = resolver
                .get_url(encoding.into(), network_id.into())
                .await
                .map_err(|err| PyException::new_err(err.to_string()))?;
            Ok(url)
        })
    }

    // fn connect() TODO
}

impl From<PyResolver> for Resolver {
    fn from(resolver: PyResolver) -> Self {
        resolver.0
    }
}
