import { applyMiddleware, createStore } from 'redux';
import logger from 'redux-logger';
import thunk from 'redux-thunk';
import reducer from './Reducer';

export default function configureStore() {
    let store = process.env.NODE_ENV !== 'production' ? 
                createStore(reducer, applyMiddleware(thunk, logger)) :
                createStore(reducer, applyMiddleware(thunk));
    return store;
}
