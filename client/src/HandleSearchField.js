import React, { Component } from 'react';
import { connect } from 'react-redux';
import * as C from './Const'

import { withStyles } from '@material-ui/core/styles';
import Paper from '@material-ui/core/Paper';
import InputBase from '@material-ui/core/InputBase';
import IconButton from '@material-ui/core/IconButton';
import SearchIcon from '@material-ui/icons/Search';

import { setHandleSearchFieldValue,
         setDrawerMenuSelection,
         setViewedHandle,
         queryGetFardelsFromHandle,
         setSelectedFardel,
       } from './store/Actions';

const useStyles = theme => ({
  root: {
    padding: '2px 4px',
    display: 'flex',
    alignItems: 'center',
    width: 260,
  },
  input: {
    marginLeft: theme.spacing(1),
    flex: 1,
  },
  iconButton: {
    padding: 10,
  },
  divider: {
    height: 28,
    margin: 4,
  },
});

class HandleSearchField extends Component {

  searchFieldInputHandler(event) {
    const { setHandleSearchFieldValue } = this.props;

    if (event.target.value === '' || event.target.value) {
      setHandleSearchFieldValue(event.target.value);
    }
  }

  handleSearchSubmit(event) {
    const { queryGetFardelsFromHandle, 
      handleSearchFieldValue, 
      setDrawerMenuSelection, 
      setViewedHandle,
      setSelectedFardel, } = this.props;

    event.preventDefault();
    setViewedHandle(handleSearchFieldValue);
    setDrawerMenuSelection(C.DRAWER_MENU_VIEW_HANDLE);
    queryGetFardelsFromHandle(handleSearchFieldValue, 0, 10);
    setSelectedFardel(-1);
  }

  render() {
    const { classes } = this.props;
    return (
      <Paper component="form" className={classes.root} onSubmit={(e) => this.handleSearchSubmit(e)}>
        @
        <InputBase
          className={classes.input}
          placeholder="View new handle"
          inputProps={{ 'aria-label': 'search handle' }}
          onChange={(e) => this.searchFieldInputHandler(e)}
        />
        <IconButton type="submit" className={classes.iconButton} aria-label="search">
          <SearchIcon />
        </IconButton>
      </Paper>
    );
  }
}

const mapStateToProps = (state) => {
  return {
    keplrEnabled: state.keplrEnabled,
    handleSearchFieldValue: state.handleSearchFieldValue,
  }
};

const mapDispatchToProps = (dispatch) => {
  return ({
    queryGetFardelsFromHandle: (handle, page, pageSize) =>
        { dispatch(queryGetFardelsFromHandle(handle, page, pageSize)) },
    setHandleSearchFieldValue: (value) => { dispatch(setHandleSearchFieldValue(value)) },
    setDrawerMenuSelection: (drawerMenuSelection) => { dispatch(setDrawerMenuSelection(drawerMenuSelection)) },
    setViewedHandle: (viewedHandle) => { dispatch(setViewedHandle(viewedHandle)) },
    setSelectedFardel: (selectedFardel) => { dispatch(setSelectedFardel(selectedFardel)) },
  });
}

export default connect(mapStateToProps, mapDispatchToProps)(withStyles(useStyles)(HandleSearchField));
